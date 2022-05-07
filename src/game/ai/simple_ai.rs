use super::super::{GameAction, Node, Point, PointSet, Team};
use super::pathfinding;

pub(super) fn generate_enemy_ai_actions<C: FnMut(GameAction)>(
    mut node: Node,
    team_sprites: Vec<usize>,
    mut collect: C,
) {
    // TODO handle partial states
    // Currently just move all sprites to the right
    let mut game_actions: Vec<GameAction> = Vec::new();
    for sprite_key in team_sprites {
        simple_greedy_attack(sprite_key, &node, |action| {
            log::debug!("CALLED COLLECT {:?}", action);
            collect(action.clone());
            (&mut game_actions).push(action);
        });
        for action in game_actions.iter() {
            node.apply_action(action)
                .expect("Unexpected error applying generated action");
        }
        game_actions.clear();

        // TODO apply new_actions to Node so multiple sprite keys can interop
    }
}

pub fn simple_greedy_attack<C: FnMut(GameAction)>(sprite_key: usize, node: &Node, mut collect: C) {
    /*

    Current limitations:
        * Not fast, really
        * Frontloaded
        * Pathfinding might take long routes
        * Only does attack actions
        * Does not move if not in range to attack

    Algorithm:
        let attack = Get sprite actions, find the highest range one that targets enemies. Damage doesn't matter.
        For each sprite on the enemy team, find any that are within (movement + attack.range)
            See if it is possible to move within attack.range of that sprite
            If it is possible, add it to the list of possible targets.
            Pick the sprite with the smallest size
        A better greedy algorithm would find the action with the largest range but also most damage.
        A better greedy attack would try to find a sprite in range of the strongest attack first.
        A better greedy attack might prioritize killing the most dangerous sprite.
        A better greedy attack might try to figure out a way to stay out of range of enemy attacks
        A better greedy algorithm would take into account conditions necessary to use actions, but for now
        we just have to not put actions like that on the AI.
    */

    node.with_sprite(sprite_key, |sprite| {
        collect(GameAction::activate_sprite(sprite_key));

        if let Some((action_index, preferred_action)) = sprite
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
                .with_sprite(sprite_key, |sprite| sprite.possible_moves())
                .unwrap();
            let strike_spaces = get_points_within_x_of_enemy_team(node, range) & possible_moves;

            // For now just take whatever one is first
            if let Some(target) = strike_spaces.into_set().iter().next() {
                move_to_target(sprite_key, *target, node, &mut collect);
                let strike_range = PointSet::range_of_pt(*target, range, node.bounds());
                let sprite_target_keys = node.filtered_sprite_keys(|_, sprite| {
                    sprite.team() == Team::PlayerTeam && strike_range.contains(sprite.head())
                });
                // For now just pick the first one
                let chosen_target = *sprite_target_keys
                    .get(0)
                    .expect("Weird if there are no sprites within range of the calculated target");
                let chosen_target_pt = node
                    .with_sprite(chosen_target, |sprite| sprite.head()) // FIXME The head is not the only targetable piece of the player
                    .expect("Chosen target should have a head");

                collect(GameAction::take_sprite_action(
                    action_index,
                    chosen_target_pt,
                ));
            } else {
                // For now, do nothing. In the future, we might:
                // Pathfind towards /closest/ enemy
                // Pathfind towards where the access points were defined
                // Patrol?
                // Maybe I'll add metadata to each file to add hints for the AI.
                // AI will probably be massively configurable per node
                collect(GameAction::deactivate_sprite());
            }
        } else {
            collect(GameAction::deactivate_sprite());
        }
    })
    .expect("Somehow we got called with an invalid sprite key")
}

fn get_points_within_x_of_enemy_team(node: &Node, range: usize) -> PointSet {
    // TODO Bugfix: The head is not the only targetable piece of the player
    let bounds = node.bounds();
    let pts: Vec<_> = node
        .sprite_keys_for_team(Team::PlayerTeam)
        .iter()
        .map(|key| {
            let head = node.with_sprite(*key, |sprite| sprite.head()).expect(
                "An immutable node reference just provided these keys, they should be valid",
            );
            PointSet::range_of_pt(head, range, bounds)
        })
        .collect();
    PointSet::merge(pts)
}

fn move_to_target<C: FnMut(GameAction)>(
    sprite_key: usize,
    target: Point,
    node: &Node,
    mut collect: C,
) {
    for dir in pathfinding::find_any_path_to_point(sprite_key, target, node)
        .expect("TODO What if pathfinding fails?") // TODO It shouldn't... But what then?
        .into_iter()
    {
        collect(GameAction::move_active_sprite(vec![dir]));
    }
}
