use super::super::{Direction, Node, PointSet, Team};
use super::{EnemyAiAction};

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
        if let Some((action_index, preferred_action)) = sprite.actions()
            .iter()
            .enumerate()
            .filter(|(_, action)|action.can_target_enemy())
            .max_by_key(|(_, action)|action.range()) {


            // In the future, might be able to get a more accurate point set
            let pt_set = PointSet::range_of_pt(sprite.head(), preferred_action.range().map_or(0, NonZeroUsize::get) + sprite.moves() , node.bounds());


            let first_round_elimination = node.filtered_sprite_keys(|_, sprite| {
                sprite.team() != Team::EnemyTeam && pt_set.contains(sprite.head())
            });
            let mut actions = Vec::new();
            actions.push(EnemyAiAction::PerformNoAction);
            actions.push(EnemyAiAction::MoveSprite(Direction::East));
            actions.push(EnemyAiAction::ActivateSprite(sprite_key));

            actions
        } else {
            // I don't have any attacks. I guess do nothing for now.
            vec![
                EnemyAiAction::PerformNoAction,
                EnemyAiAction::ActivateSprite(sprite_key),
            ]
        }
    }).expect("Somehow we got called with an invalid sprite key")
}
