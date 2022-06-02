//! Utility module for helping with pathfinding for AI

use crate::{Direction, Node, Point};
use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::VecDeque;

/// Highly unoptimized and potentially buggy algorithm, significant slow down at speeds around 12.
/// ... But hey, it does what it says on the box.
pub fn find_any_path_to_point(
    curio_key: usize,
    target: Point,
    node: &Node,
) -> Option<Vec<Direction>> {
    node.with_curio(curio_key, |curio| {
        let head = curio.head();
        if head == target {
            return Some(Vec::new());
        }
        let mut visited = vec![vec![0; node.height()]; node.width()];
        visited[head.0][head.1] = curio.moves();
        let first_dir = prime_direction_between_points(head, target);
        let dir_iter = std::iter::successors(first_dir, |dir| Some(dir.clockwise())).take(4);
        for dir in dir_iter {
            if let Some(path) = find_any_path_to_point_biased(
                curio_key,
                head,
                target,
                node,
                dir,
                &mut visited,
                curio.moves(),
            ) {
                return Some(path.into_iter().rev().collect());
            }
        }
        None
    })
}

fn find_any_path_to_point_biased(
    curio_key: usize,
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
        || node.grid().item_key_at(move_pt) == Some(curio_key)
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
                curio_key,
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

// Chooses a direction so that we move in a given direction based on what sector the target is in
// TODO Use something like this for click-based movement as well
fn prime_direction_between_points(src: Point, target: Point) -> Option<Direction> {
    match [target.0.cmp(&src.0), target.1.cmp(&src.1)] {
        [Equal, Less] | [Greater, Less] => Some(Direction::North),
        [Greater, Equal] | [Greater, Greater] => Some(Direction::East),
        [Equal, Greater] | [Less, Greater] => Some(Direction::South),
        [Less, Equal] | [Less, Less] => Some(Direction::West),
        [Equal, Equal] => None,
    }
}
