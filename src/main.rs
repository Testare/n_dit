use crossterm::{
    event, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand, Result,
};
use n_dit::grid_map::Point;
use itertools::Itertools;
use std::cmp::min;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::io::{stdout, Write};
use std::ops::AddAssign;

const DRAW_TYPE: DrawType = DrawType::CrossLink2;
const INTERSECTION_CHAR: [char; 16] = [
    ' ', '?', '?', '└', '?', '│', '┌', '├', '?', '┘', '─', '┴', '┐', '┤', '┬', '┼',
];

// type Point = (usize, usize);
type PieceRef = usize;

struct Bounds {
    height: usize,
    width: usize,
}

#[derive(PartialEq, Eq)]
struct Sprite {
    display: String,
    team: usize,
    moved: bool,
    head: Point,
}

// Represent things in the field
// Perhaps we change from enum to struct
enum Piece {
    Program(Sprite),
    Mon(u32),
}

// Represents a location in the node
// PIECE
enum Square {
    AccessPoint,
    Zero,
    One,
    Token {
        piece_index: usize,
        next: Option<Point>,
    },
}

enum Direction {
    North = 0,
    East,
    South,
    West,
}

struct Node {
    pieces: Vec<Piece>,
    grid: Vec<Vec<Square>>,
    bounds: Bounds,
}

struct SpriteIter<'a> {
    node: &'a Node,
    point: Option<Point>,
}

impl<'a> SpriteIter<'a> {
    fn new(piece_id: PieceRef, node: &'a Node) -> Self {
        SpriteIter {
            node,
            point: node.piece_from_id(piece_id).and_then(|piece| match piece {
                Piece::Program(sprite) => Some(sprite.head),
                _ => None,
            }),
        }
    }
}

impl<'a> Iterator for SpriteIter<'a> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let next_pt = self.point?;
        let sqr = self.node.square(&next_pt);
        if let Square::Token { next, .. } = sqr {
            self.point = *next;
        } else {
            self.point = None;
        }
        Some(next_pt)
    }
}

// Concern: If I do copies, that is a lot of data copying going on
//
// If I don't, "undo" becomes difficult to figure out.
// Could I wrap everything in Rc?
//
// Rust doesn't have much as far as immutable collections
//

impl Node {
    fn add_square(&self, piece_id: PieceRef, square: Point) {}

    fn piece_from_id(&self, piece_id: PieceRef) -> Option<&Piece> {
        self.pieces.get(piece_id)
    }

    fn piece_at_point(&self, point: &Point) -> Option<&Piece> {
        match self.square(point) {
            Square::Token { piece_index, .. } => self.pieces.get(*piece_index),
            _ => None,
        }
    }

    fn square(&self, (x, y): &Point) -> &Square {
        &self.grid[*x][*y]
    }

    fn move_actor(&self, direction: Direction) -> bool {
        false
    }
}

impl Direction {
    /// Moves point in direction within `bounds`. If the move is out of bounds, returns None.
    pub fn bump(&self, (x, y): &Point, bounds: &Bounds) -> Option<Point> {
        match self {
            Direction::North => y.checked_sub(1).map(|y| (*x, y)),
            Direction::East => {
                if x + 1 == bounds.width {
                    None
                } else {
                    Some((x + 1, *y))
                }
            }
            Direction::South => {
                if y + 1 == bounds.height {
                    None
                } else {
                    Some((*x, y + 1))
                }
            }
            West => x.checked_sub(1).map(|x| (x, *y)),
        }
    }
}

#[derive(PartialEq, Eq)]
enum DrawType {
    BorderlessLink = 0,
    CrossLink1,
    CrossLink2,
    CrossLink3,
    DotLink,
}

#[derive(PartialEq, Eq)]
enum FillMethod {
    Brackets = 0,
    NoFill = 1,
    HeadCopy = 2,
    DotFill = 3,
}

#[derive(PartialEq, Eq)]
enum BorderType {
    Borderless = 0,
    Bordered = 1,
    Linked = 2,
}

impl BorderType {
    fn horizontal_border(&self, draw_type: DrawType) -> &'static str {
        match self {
            BorderType::Borderless => "  ",
            BorderType::Bordered => "──",
            BorderType::Linked => match draw_type {
                DrawType::BorderlessLink => "  ",
                DrawType::CrossLink1 => "╫─",
                DrawType::CrossLink2 => "┤├",
                DrawType::CrossLink3 => "┼┼",
                DrawType::DotLink => "..",
            },
        }
    }

    fn vertical_border(&self, draw_type: DrawType) -> char {
        match self {
            BorderType::Borderless => ' ',
            BorderType::Bordered => '│',
            BorderType::Linked => match draw_type {
                DrawType::BorderlessLink => ' ',
                DrawType::DotLink => '.',
                _ => '╪',
            },
        }
    }
}

fn main() -> Result<()> {
    execute!(
        stdout(),
        SetForegroundColor(Color::Blue),
        SetBackgroundColor(Color::Red),
        Print("Styled text here."),
        ResetColor
    )?;
    let m = intersection_for(&[2, 0], &[2, 0]);
    println!("Now for a region map ({})", m);
    // This is a decent proof of concept
    // To finish, will need a way to render the actual grid pieces
    // Will need a way to specify coloring/effects for different squares
    // Perhaps instead of slices of u8, they could be slices of grid pieces, with their own
    // affectations as properties
    // For instance, if the grid should be displayed in red

    let region_map = [
        &[0, 0, 0, 0, 0, 0, 0, 0, 0][..],
        &[0, 0, 0, 1, 1, 1, 0, 0, 0][..],
        &[0, 0, 2, 1, 4, 4, 1, 0, 0][..],
        &[0, 1, 2, 2, 2, 1, 1, 1, 0][..],
        &[0, 0, 1, 2, 2, 1, 1, 0, 0][..],
        &[0, 0, 0, 1, 2, 1, 0, 0, 0][..],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0][..],
    ];
    let drawing = region_map_drawing(&region_map[..]);
    for string in drawing.iter() {
        println!("/ {} /", string);
    }

    Ok(())
}

// Does not handle adding the margin here
fn region_map_drawing(map: &[&[u8]]) -> Vec<String> {
    let border_lines = map.windows(2).map(|vertical_slice| {
        if let [top, bottom] = vertical_slice {
            let mut intersections = top
                .windows(2)
                .zip(bottom.windows(2))
                .map(|(top2, bottom2)| {
                    intersection_for(
                        <&[u8; 2]>::try_from(top2).unwrap(),
                        <&[u8; 2]>::try_from(bottom2).unwrap(),
                    )
                });
            let first_corner = String::from(intersections.next().unwrap()); // Need to figure out what to do in case of no intersections later
            let horizontal_borders = top
                .iter()
                .zip(bottom.iter())
                .skip(1)
                .map(|(top, bottom)| border_type(*top, *bottom).horizontal_border(DRAW_TYPE));

            horizontal_borders.zip(intersections).fold(
                first_corner,
                |mut acm, (hb, intersection)| {
                    acm.push_str(hb);
                    acm.push(intersection);
                    acm
                },
            )
        } else {
            panic!("Unexpected behavior from slice windows function.")
        }
    });
    let rows = map.iter().skip(1).map(|row| {
        let mut border_chars = row.windows(2).map(|slice| {
            if let [left, right] = slice {
                border_type(*left, *right).vertical_border(DRAW_TYPE)
            } else {
                panic!("Unexpected behavior from slice windows function.")
            }
        });
        let first = border_chars.next().unwrap();
        border_chars.fold(String::from(first), |mut acm, border_char| {
            acm.push_str("  ");
            acm.push(border_char);
            acm
        })
    });
    border_lines.interleave_shortest(rows).collect()
}

/*
│	┤	╡	╢	╖	╕	╣	║	╗	╝	╜	╛	┐
C	└	┴	┬	├	─	┼	╞	╟	╚	╔	╩	╦	╠	═	╬	╧
D	╨	╤	╥	╙	╘	╒	╓	╫	╪	┘	┌
*/

fn intersection_for(top: &[u8; 2], bottom: &[u8; 2]) -> char {
    #[inline]
    fn border_type_bit(one: u8, the_other: u8) -> usize {
        if DRAW_TYPE == DrawType::BorderlessLink {
            usize::from(border_type(one, the_other) == BorderType::Bordered)
        } else {
            usize::from(border_type(one, the_other) != BorderType::Borderless)
        }
    }
    let north = border_type_bit(top[0], top[1]);
    let east = border_type_bit(top[1], bottom[1]) << 1;
    let south = border_type_bit(bottom[0], bottom[1]) << 2;
    let west = border_type_bit(top[0], bottom[0]) << 3;

    // print!("N:{} E:{} S:{} W:{}", north, east, south, west);

    INTERSECTION_CHAR[north | east | south | west]
}

fn border_type(lhs: u8, rhs: u8) -> BorderType {
    if lhs != rhs {
        BorderType::Bordered
    } else {
        match lhs {
            0 => BorderType::Borderless,
            1 => BorderType::Bordered,
            _ => BorderType::Linked,
        }
    }
}
/*
 * For figuring out borders, Grid should be converted into a RegionMap, where each cell is
 * denominated with a number.
 *
 * 0 0 0 0 0 0
 * 0 0 1 1 0 0
 * 0 1 1 2 2 0
 * 0 1 3 2 1 0
 * 0 0 1 1 0 0
 * 0 0 0 0 0 0
 *
 * (A margin of 0's will be automatically added)
 *
 * Then we pass in a pairing chart to figure out the relationship between borders
 * 0 -> 0: Borderless
 * 0 -> 1: Bordered
 * 0 -> 2: Bordered
 * 0 -> 3: Bordered
 * 1 -> 1: Bordered
 * 1 -> 2: Bordered
 * 1 -> 3: Bordered
 * 2 -> 2: Linked
 * 2 -> 3: Bordered
 * 3 -> 3: Linked
 *
 * Actually, this looks like it could be simplified as a map of how a number should border itself:
 * 0: Borderless
 * 1: Bordered
 * 2: Linked
 * 3: Linked
 *
 *
 * Then we pass a pair of slices of 2
 *
 * Actually... It's pretty much "0 is borderless, 1 is bordered, all else is linked
 *
 * All else is bordered
 *
 **/