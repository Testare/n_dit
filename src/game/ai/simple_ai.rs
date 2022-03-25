use super::super::{Direction, Node, Point, PointSet, Team};
use super::pathfinding;
use super::EnemyAiAction;

use std::num::NonZeroUsize;

pub(super) fn generate_enemy_ai_actions(
    node: Node,
    team_sprites: Vec<usize>,
) -> Vec<EnemyAiAction> {
    // Currently just move all sprites to the right
    let mut actions = Vec::new();
    for sprite_key in team_sprites {
        let mut new_actions = simple_greedy_attack(sprite_key, &node);
        // TODO apply new_actions to Node so multiple sprite keys can interop
        actions.append(&mut new_actions);
    }
    actions
}

pub fn simple_greedy_attack(sprite_key: usize, node: &Node) -> Vec<EnemyAiAction> {
    /*
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
                let mut enemy_actions = move_to_target(sprite_key, *target, node);
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
                enemy_actions.push(EnemyAiAction::PerformAction(action_index, chosen_target_pt));

                enemy_actions.into_iter().rev().collect()
            } else {
                // For now, do nothing. In the future, we might:
                // Pathfind towards /closest/ enemy
                // Pathfind towards where the access points were defined
                // Patrol?
                // Maybe I'll add metadata to each file to add hints for the AI.
                // AI will probably be massively configurable per node
                do_nothing(sprite_key)
            }
        } else {
            do_nothing(sprite_key)
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

fn do_nothing(sprite_key: usize) -> Vec<EnemyAiAction> {
    vec![
        EnemyAiAction::PerformNoAction,
        EnemyAiAction::ActivateSprite(sprite_key),
    ]
}

fn move_to_target(sprite_key: usize, target: Point, node: &Node) -> Vec<EnemyAiAction> {
    let mut enemy_actions = vec![EnemyAiAction::ActivateSprite(sprite_key)];
    let movements = pathfinding::find_any_path_to_point(sprite_key, target, node)
        .into_iter()
        .map(|dir| EnemyAiAction::MoveSprite(dir));
    enemy_actions.extend(movements);
    enemy_actions
}
