//! Utility module for helping with pathfinding for AI

use crate::{Direction, Node, Point};
use std::collections::VecDeque;

/// Highly unoptimized and potentially buggy algorithm, significant slow down at speeds around 12.
/// ... But hey, it does what it says on the box.
pub fn find_any_path_to_point(
    sprite_key: usize,
    target: Point,
    node: &Node,
) -> Option<Vec<Direction>> {
    node.with_sprite(sprite_key, |sprite| {
        let head = sprite.head();
        if head == target {
            return Some(Vec::new());
        }
        let mut visited = vec![vec![0; node.height()]; node.width()];
        visited[head.0][head.1] = sprite.moves();
        // TODO Intelligently determine starter direction
        for dir in Direction::EVERY_DIRECTION {
            if let Some(path) = find_any_path_to_point_biased(
                sprite_key,
                head,
                target,
                node,
                dir,
                &mut visited,
                sprite.moves(),
            ) {
                return Some(path.into_iter().rev().collect());
            }
        }
        None
    })
}

fn find_any_path_to_point_biased(
    sprite_key: usize,
    current_pt: Point,
    target: Point,
    node: &Node,
    dir: Direction,
    visited: &mut Vec<Vec<usize>>,
    remaining_moves: usize,
) -> Option<Vec<Direction>> {
    let move_pt = (current_pt + dir)?;
    if move_pt.0 >= node.width() || move_pt.1 >= node.height() {
        return None;
    }
    if visited[move_pt.0][move_pt.1] >= remaining_moves {
        return None;
    }
    visited[move_pt.0][move_pt.1] = remaining_moves;

    if move_pt == target {
        return Some(vec![dir]);
    } else if remaining_moves == 1 {
        return None;
    } else if node.grid().square_is_free(move_pt)
        || node.grid().item_key_at(move_pt) == Some(sprite_key)
    {
        let mut possible_dirs = VecDeque::from([dir, dir.clockwise(), dir.clockwise().flip()]); // Pointless to backtrack
                                                                                                // If we lined up with our quary, try moving in that direction first.
        if dir.vertical() && move_pt.1 == target.1 {
            if move_pt.0 < target.0 {
                possible_dirs.push_front(Direction::East);
            } else {
                possible_dirs.push_front(Direction::West)
            }
        } else if dir.horizontal() && move_pt.0 == target.0 {
            if move_pt.1 < target.1 {
                possible_dirs.push_front(Direction::South)
            } else {
                possible_dirs.push_front(Direction::North)
            }
        }

        for new_dir in possible_dirs {
            if let Some(mut path_from_here) = find_any_path_to_point_biased(
                sprite_key,
                move_pt,
                target,
                node,
                new_dir,
                visited,
                remaining_moves - 1,
            ) {
                path_from_here.push(dir);
                return Some(path_from_here);
            }
        }
    }
    None
}
