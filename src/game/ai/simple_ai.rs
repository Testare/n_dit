use super::super::{Node, NodeChange, Point, PointSet, Team};
use super::pathfinding;

pub(super) fn generate_enemy_ai_actions<C: FnMut(NodeChange)>(
    mut node: Node,
    team_curios: Vec<usize>,
    mut collect: C,
) {
    // Currently just move all curios to the right
    let mut changes: Vec<NodeChange> = Vec::new();
    for curio_key in team_curios {
        simple_greedy_attack(curio_key, &node, |change| {
            log::debug!("CALLED COLLECT {:?}", change);
            collect(change);
            (&mut changes).push(change);
        });
        for change in changes.iter() {
            change
                .apply(&mut node)
                .expect("Unexpected error applying generated action");
        }
        changes.clear();
    }
    collect(NodeChange::FinishTurn)
    // TODO add FinishTurn
}

pub fn simple_greedy_attack<C: FnMut(NodeChange)>(curio_key: usize, node: &Node, mut collect: C) {
    /*

    Current limitations:
        * Not fast, really
        * Frontloaded
        * Pathfinding might take long routes
        * Only does attack actions
        * Does not move if not in range to attack

    Algorithm:
        let attack = Get curio actions, find the highest range one that targets enemies. Damage doesn't matter.
        For each curio on the enemy team, find any that are within (movement + attack.range)
            See if it is possible to move within attack.range of that curio
            If it is possible, add it to the list of possible targets.
            Pick the curio with the smallest size
        A better greedy algorithm would find the action with the largest range but also most damage.
        A better greedy attack would try to find a curio in range of the strongest attack first.
        A better greedy attack might prioritize killing the most dangerous curio.
        A better greedy attack might try to figure out a way to stay out of range of enemy attacks
        A better greedy algorithm would take into account conditions necessary to use actions, but for now
        we just have to not put actions like that on the AI.
    */

    node.with_curio(curio_key, |curio| {
        collect(NodeChange::ActivateCurio(curio_key));

        if let Some((action_index, preferred_action)) = curio
            .actions()
            .iter()
            .enumerate()
            .filter(|(_, action)| action.can_target_enemy() && action.range().is_some())
            .max_by_key(|(_, action)| action.range())
        {
            let range = preferred_action
                .range()
                .expect("Actions with no range should've been filtered")
                .get();

            // In the future, might be able to get a more accurate point set
            let possible_moves = node
                .with_curio(curio_key, |curio| curio.possible_moves())
                .unwrap();
            let strike_spaces = get_points_within_x_of_enemy_team(node, range) & possible_moves;

            // For now just take whatever one is first
            if let Some(target) = strike_spaces.into_set().iter().next() {
                move_to_target(curio_key, *target, node, &mut collect);
                let strike_range = PointSet::range_of_pt(*target, range, node.bounds());
                let curio_target_keys = node.filtered_curio_keys(|_, curio| {
                    curio.team() == Team::PlayerTeam && strike_range.contains(curio.head())
                });
                // For now just pick the first one
                let chosen_target = *curio_target_keys
                    .get(0)
                    .expect("Weird if there are no curios within range of the calculated target");
                let chosen_target_pt = node
                    .with_curio(chosen_target, |curio| curio.head()) // FIXME The head is not the only targetable piece of the player
                    .expect("Chosen target should have a head");

                collect(NodeChange::TakeCurioAction(action_index, chosen_target_pt));
            } else {
                // For now, do nothing. In the future, we might:
                // Pathfind towards /closest/ enemy
                // Pathfind towards where the access points were defined
                // Patrol?
                // Maybe I'll add metadata to each file to add hints for the AI.
                // AI will probably be massively configurable per node
                collect(NodeChange::DeactivateCurio);
            }
        } else {
            collect(NodeChange::DeactivateCurio);
        }
    })
    .expect("Somehow we got called with an invalid curio key")
}

fn get_points_within_x_of_enemy_team(node: &Node, range: usize) -> PointSet {
    // TODO Bugfix: The head is not the only targetable piece of the player
    let bounds = node.bounds();
    let pts: Vec<_> = node
        .curio_keys_for_team(Team::PlayerTeam)
        .iter()
        .map(|key| {
            let head = node.with_curio(*key, |curio| curio.head()).expect(
                "An immutable node reference just provided these keys, they should be valid",
            );
            PointSet::range_of_pt(head, range, bounds)
        })
        .collect();
    PointSet::merge(pts)
}

fn move_to_target<C: FnMut(NodeChange)>(
    curio_key: usize,
    target: Point,
    node: &Node,
    mut collect: C,
) {
    for dir in pathfinding::find_any_path_to_point(curio_key, target, node)
        .expect("TODO What if pathfinding fails?") // TODO It shouldn't... But what then?
        .into_iter()
    {
        collect(NodeChange::MoveActiveCurio(dir));
    }
}
