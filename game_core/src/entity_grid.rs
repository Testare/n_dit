//! Documentation might be outdated! This was recently migrated from a very 
//! different implementation.

use bitvec::{slice::BitSlice, vec::BitVec};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, iter::Rev, vec::IntoIter};
use bevy::reflect::{FromReflect, Reflect};
use crate::prelude::*;
pub mod commands;

// Potential future developments:
// * removing squares from the middle of an Item
// ^ Forcibly adding a square to an item, removing squares from other entries and opening closed
// squares if necessary
// * modify put_item, push_front, and push_back to take a point OR iterator of points.
// * take_entries to remove multiple entries.
// * put_entries variant that doesn't add anything if any entries are invalid
// * size() -> occupied squares, capacity() -> free squares
// * a visual map that indicates square order (Perhaps as a tuple of (item_key, order)?
// * ability to adjust EntityGrid size.
// * moves direction from head (move/grow)
// * push_back logic to match push_front logic when the square is already occupied by the item.

/// Represents a point of space that may contain a square.

/// Internal representation of available space. Contains:
/// * A reference to its location on the map
/// * An id for an item in the containing EntityGrid, if the square is occupied.
/// * A reference to the next square occupied by the item, if any.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromReflect, Reflect)]
pub struct Square {
    item: Option<Entity>,
    next: Option<UVec2>,
    location: UVec2,
}

/// A very specialized data structure. Contains a representation of a grid. Items in the map must
/// have at least one square of representation in the grid, possibly more. These squares are
/// ordered. A square in the grid must be "open" in order to contain an item.

#[derive(Clone, Component, Debug, PartialEq, Eq, Serialize, Deserialize, FromReflect, Reflect)]
pub struct EntityGrid {
    width: u32,
    height: u32,
    entries: HashMap<Entity, UVec2>,
    grid: Vec<Vec<Option<Square>>>, // None = closed. No grid to be. At no point should a square be inserted here from outside
}

#[derive(Clone, Debug)]
/// An iterator over the squares of the grid map for a given item.
pub struct SquareIter<'a> {
    map: &'a EntityGrid,
    next: Option<UVec2>,
}

#[derive(Debug)]
/// A mutable iterator over the suqares of the grid map for a given item. Only to be used
/// internally, as squares should not be directly mutable externally.
struct SquareIterMut<'a> {
    map: &'a mut EntityGrid,
    next: Option<UVec2>,
}

impl Square {
    /// Creates an empty square. Should not be used outside of EntityGrid
    fn new(location: UVec2) -> Self {
        Square {
            item: None,
            next: None,
            location,
        }
    }

    /// Gets the key to the item this square contains.
    pub fn item_key(&self) -> Option<Entity> {
        self.item
    }

    /// Returns the point of the next square after this one linked to the same item.
    pub fn next(&self) -> Option<UVec2> {
        self.next
    }

    /// The location of this square on the grid.
    pub fn location(&self) -> UVec2 {
        self.location
    }

    /// Empties the square, clearing both item and next.
    fn clear(&mut self) {
        self.item = None;
        self.next = None;
    }

    /// Sets the item key
    fn set_item_key<E: Into<Option<Entity>>>(&mut self, item: E) {
        self.item = item.into()
    }

    /// Sets point of next square on grid linked to the same item.
    /// Should never be [`Some`] when `item_key` is [`None`]
    fn set_next<P: Into<Option<UVec2>>>(&mut self, point: P) {
        self.next = point.into()
    }
}

/// Convenience trait to allow Square to passed to [`EntityGrid::item()`]
/// Simply converts a Square to the optional item_key.
impl From<Square> for Option<Entity> {
    fn from(sqr: Square) -> Option<Entity> {
        sqr.item_key()
    }
}

impl EntityGrid {
    /// A representation of closed and open squares, though no width/height information encoded.
    /// Does not support maps with width/height greater u16::MAX
    pub fn shape_bitvec(&self) -> BitVec<u8> {
        let height: [u8; 2] = (self.height() as u16).to_le_bytes();
        let width: [u8; 2] = (self.width() as u16).to_le_bytes();
        let squarebits: BitVec<u8> =  self.grid
            .iter()
            .flat_map(|col| col.iter().map(|sqr| sqr.is_some()))
            .collect();

        let mut bitvec = BitVec::<u8>::new();
        bitvec.extend_from_raw_slice(&width[..]);
        bitvec.extend_from_raw_slice(&height);
        bitvec.extend(squarebits);
        bitvec
    }

    pub fn shape_string_base64(&self) -> String {
        base64::encode(self.shape_bitvec().into_vec())
    }

    /// Closes a square. Returns false if it is already closed, is occupied, or it is out of bounds.
    pub fn close_square(&mut self, pt: UVec2) -> bool {
        if self.square_is_free(pt) {
            self.grid[pt.x as usize][pt.y as usize] = None;
            true
        } else {
            false
        }
    }

    /// Determine if a key is actually
    pub fn contains_key(&self, item_key: Entity) -> bool {
        self.entries.contains_key(&item_key)
    }

    /// Determins if a point is within bounds of the EntityGrid
    pub fn contains_point(&self, UVec2 {x, y}: UVec2) -> bool {
        x < self.width && y < self.height
    }

    /// Returns the front point where the given item is in the grid
    pub fn head(&self, item_key: Entity) -> Option<UVec2> {
        self.entries.get(&item_key).copied()
    }

    /// Returns the back point where the given item is in the grid
    pub fn back(&self, item_key: Entity) -> Option<UVec2> {
        self.square_iter(item_key).map(|sqr| sqr.location()).last()
    }

    /// Returns the height of the map
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns a reference to the item at the given point
    pub fn item_at(&self, pt: UVec2) -> Option<Entity> {
        self.square_ref(pt)?.item_key()
    }

    /// Returns a list of keys for all entries contained in the grid.
    ///
    /// There is no guarantee to order.
    pub fn entities(&self) -> Vec<Entity> {
        self.entries.keys().copied().collect()
    }

    /// Returns a list of keys for all entries contained in the grid that match the criteria
    /// of the predicate.
    ///
    /// Predicate function takes two parameters: The key and an immutable reference to the item.EnemyAi
    ///
    /// Result is a list of keys.
    ///
    /// There is no guarantee to order.
    pub fn filtered_keys<P: Fn(Entity) -> bool>(&self, predicate: P) -> Vec<Entity> {
        self.entries
            .keys()
            .filter(|(key)| predicate(**key))
            .copied()
            .collect()
    }

    pub fn from_shape_string(shape: &str) -> Result<Self, base64::DecodeError> {
        let bits: Vec<u8> = base64::decode(shape)?;
        let bitvec = BitVec::<u8>::from_vec(bits);
        Ok(Self::from_shape_bitslice(bitvec.as_bitslice()))
    }

    /// Creates a base grid_map from a shape string
    /// 
    pub fn from_shape_bitslice(bits: &BitSlice<u8>) -> Self {
        let (hw, squarebits) = bits.split_at(32);
        let (wbits, hbits) = hw.split_at(16);
        let mut wbytes: [u8; 2] = Default::default();
        let mut hbytes: [u8; 2] = Default::default();
        wbytes.copy_from_slice(wbits.to_bitvec().as_raw_slice());
        hbytes.copy_from_slice(hbits.to_bitvec().as_raw_slice());
        let width = <u16>::from_le_bytes(wbytes) as u32;
        let height = <u16>::from_le_bytes(hbytes) as u32;

        let grid: Vec<Vec<Option<Square>>> = squarebits
            .chunks(height as usize)
            .enumerate()
            .map(|(x, col)| {
                col.iter()
                    .enumerate()
                    .map(|(y, bit)| bit.then(|| Square::new((x as u32, y as u32).into())))
                    .collect()
            })
            .take(width as usize)
            .collect();

        EntityGrid {
            height,
            entries: HashMap::new(),
            grid,
            width,
        }
    }

    /// Returns the number of entries currently stored in the grid. Unrelated to the dimensions of
    /// the grid or the amount of square each item takes.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns if the grid is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of squares an item takes up
    pub fn len_of(&self, item_key: Entity) -> usize {
        self.square_iter(item_key).count()
    }

    /// Creates a new grid map of certain dimensions. By default all squares will be closed,
    /// and need to be opened manually with [`open_square`](Self::open_square). For this reason,
    /// it might be more convenient to create with [`EntityGrid::from<Vec<Vec<bool>>>()`].
    pub fn new(width: u32, height: u32) -> Self {
        let grid = (0..width).into_iter().map(|_| vec![None; height as usize]).collect();

        EntityGrid {
            height,
            entries: HashMap::new(),
            grid,
            width,
        }
    }

    /// Returns a visualization of the grid using 0's for blocked squares, 1's for open but empty
    /// squares, and numbers for each entity
    ///
    /// This map does not indicate what order the squares are in, and so could
    /// not be used to reconstruct a EntityGrid even if a list of entries is provided.
    ///
    /// Each internal [`Vec<usize>`] represents a column so that the returned result can be
    /// indexed like `number_map[x][y]`.
    pub fn number_map(&self) -> Vec<Vec<usize>> {
        // TODO IMMEDIATELY Need to map entities to different numbers for this number map
        let numbers: HashMap<Entity, usize> = self.entries.iter()
            .enumerate()
            .map(|(i, (entity, _))| (*entity, i+2))
            .collect();
        self.grid
            .iter()
            .map(|col| {
                col.iter()
                    .map(|sqr_opt| sqr_opt.map(|sqr| sqr.item_key().map(|e|numbers[&e]).unwrap_or(1)).unwrap_or(0))
                    .collect()
            })
            .collect()
    }

    /// Opens a square. Returns false if it is already open or it is out of bounds, true otherwise.
    pub fn open_square(&mut self, pt: UVec2) -> bool {
        if self.square_is_closed(pt) {
            self.grid[pt.x as usize][pt.y as usize] = Some(Square::new(pt));
            true
        } else {
            false
        }
    }

    // Might be used in an optimization of the UI later, but for now we're using point_map
    // Takes a function that takes a usize that indicates a square's position in the entity's
    // queue, and then the related entity.
    pub fn point_vec<F, R>(&self, func: F) -> Vec<(UVec2, R)>
    where
        F: Fn(usize, Entity) -> R,
    {
        let mut vec: Vec<_> = self
            .entries
            .keys()
            .flat_map(|item| {
                let func_ref = &func;
                self.square_iter(*item)
                    .enumerate()
                    .map(move |(i, sqr)| (sqr.location(), func_ref(i, *item)))
            })
            .collect();
        vec.sort_by_cached_key(|(pt, _)| (pt.x, pt.y));
        vec
    }

    pub fn point_map<F, R>(&self, func: F) -> HashMap<UVec2, R>
    where
        F: Fn(usize, Entity) -> R,
    {
        self.entries
            .keys()
            .flat_map(|item| {
                let func_ref = &func;
                self.square_iter(*item)
                    .enumerate()
                    .map(move |(i, sqr)| (sqr.location(), func_ref(i, *item)))
            })
            .collect()
    }

    pub fn point_map_filtered<F, R>(&self, func: F) -> HashMap<UVec2, R>
    where
        F: Fn(usize, Entity) -> Option<R>,
    {
        self.entries
            .keys()
            .flat_map(|item| {
                let func_ref = &func;
                self.square_iter(*item)
                    .enumerate()
                    .filter_map(move |(i, sqr)| Some(sqr.location()).zip(func_ref(i, *item)))
            })
            .collect()
    }

    /// Removes an item from the last grid square this item was added to.
    ///
    /// "last" means sequentially (as in closest to the back), not chronologically.
    /// If a square was added with [`push_back`](Self::push_back), this method will remove that one
    /// before the others.
    ///
    /// If the item is completely removed from the grid, this method returns the item, else returns
    /// None.
    ///
    /// If item_key is invalid, will return None as well.
    pub fn pop_back(&mut self, item_key: Entity) -> Option<Entity> {
        let back_pt_opts = self
            .square_iter(item_key)
            .map(|sqr| sqr.location())
            .fold((None, None), |(_, acm), last| (acm, Some(last)));
        // Need to set next for the second to last item
        match back_pt_opts {
            (Some(next_back_pt), Some(back_pt)) => {
                self.square_mut(back_pt)?.set_item_key(None);
                self.square_mut(next_back_pt)?.set_next(None);
                None
            }
            (None, Some(only_pt)) => {
                self.square_mut(only_pt)?.set_item_key(None);
                self.entries.remove(&item_key).map(|_|item_key)
            }
            (None, None) => None, // There are no points here, should we panic?
            _ => panic!("Programmer error, this should not be possible"),
        }
    }

    /// Removes an item from the last `n` grid square this item was added to.
    ///
    /// "last" means sequentially (as in closest to the back), not chronologically.
    /// If a square was added with [`push_back`](Self::push_back), this method will remove that one
    /// before the others.
    ///
    /// If the item is completely removed from the grid, this method returns the item, else returns
    /// None.
    ///
    /// If item_key is invalid, will return None as well.
    pub fn pop_back_n(&mut self, item_key: Entity, n: usize) -> Option<Entity> {
        for (i, sqr) in self.square_iter_mut(item_key).rev().enumerate() {
            if i == n {
                // If there are still squares left after removing n squares
                sqr.set_next(None);
                return None; //
            } else {
                sqr.clear();
            }
        }
        self.entries.remove(&item_key).map(|_|item_key)
    }

    /// Removes an item from the first grid square this item was added to.
    ///
    /// "first" means sequentially (as in closest to the front), not chronologically.
    /// If a square was added with [`push_front`](Self::push_front), this method will remove that one
    /// before the others.
    ///
    /// If the item is completely removed from the grid, this method returns the item, else returns
    /// None.
    ///
    /// If item_key is invalid, will return None as well.
    pub fn pop_front(&mut self, item_key: Entity) -> Option<Entity> {
        let front = self.entries.get(&item_key)?;
        let square = self.square_mut(*front)?;
        let next = square.next();
        square.clear();

        match next {
            None => {
                self.entries.remove(&item_key).map(|_|item_key)
            },
            Some(next_front) => {
                *self.entries.get_mut(&item_key).unwrap() = next_front;
                None
            }
        }
    }

    /// Removes an item from the first `n` grid square this item was added to.
    ///
    /// "first" means sequentially (as in closest to the front), not chronologically.
    /// If a square was added with [`push_front`](Self::push_front), this method will remove that one
    /// before the others.
    ///
    /// If the item is completely removed from the grid, this method returns the item, else returns
    /// None.
    ///
    /// If item_key is invalid, will return None as well.
    pub fn pop_front_n(&mut self, item_key: Entity, n: usize) -> Option<Entity> {
        let head =
            self.square_iter_mut(item_key)
                .enumerate()
                .take(n + 1)
                .fold(None, |_, (i, sqr)| {
                    if i == n {
                        Some(sqr.location())
                    } else {
                        sqr.clear();
                        None
                    }
                });
        match head {
            Some(pt) => {
                *self.entries.get_mut(&item_key).unwrap() = pt;
                None
            }
            None => self.entries.remove(&item_key).map(|_| item_key),
        }
    }

    /// Adds a grid square for an item already in the [`EntityGrid`] at the back.
    ///
    /// For adding new entries to the EntityGrid, see [`put_item`](Self::put_item).
    ///
    /// Returns true if successful, returns false if the item_key doesn't
    /// correspond to an item or the square isn't free (It is closed or already
    /// occupied)
    pub fn push_back(&mut self, pt: UVec2, item_key: Entity) -> bool {
        if self.square_is_free(pt) {
            if let Some(last_sqr) = self.square_iter(item_key).last() {
                let old_last = last_sqr.location();
                self.square_mut(old_last).unwrap().set_next(pt);
                self.square_mut(pt).unwrap().set_item_key(item_key);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Adds a grid square for an item already in the [`EntityGrid`] at the front.
    ///
    /// For adding new entries to the EntityGrid, see [`put_item`](Self::put_item).
    ///
    /// If the square is already part of item in the grid map, it is moved to the front.
    ///
    /// Returns true if successful, returns false if the item_key doesn't
    /// correspond to an item, or the square isn't free (It is closed or already
    /// occupied by another item)
    pub fn push_front(&mut self, pt: UVec2, item_key: Entity) -> bool {
        if self.entries.get(&item_key).copied() == Some(pt) {
            // No operation necessary, this is already at the head
            true
        } else if self.square_is_free(pt) {
            if let Some(head) = self.entries.get_mut(&item_key) {
                let old_head = *head;
                *head = pt;
                let dest = self
                    .square_mut(pt)
                    .expect("self.square_is_free should mean that this square exists");
                dest.item = Some(item_key);
                dest.next = Some(old_head);
                true
            } else {
                false // TODO test case
            }
        } else if self.square_ref(pt).and_then(Square::item_key) == Some(item_key) {
            // Logic in here can be replaced with a call to `remove` if we ever have a case to implement this function, then moving
            // the above logic block to a private function and calling it there and here.

            let old_head = *self.entries.get(&item_key).unwrap();
            // ^ Unwrapping: Must trust all item_keys in a square. In the future, we might try branding the item_keys.
            let mut sqr_iter = self.square_iter_mut(item_key);
            let prev_sqr = sqr_iter.find(|sqr| sqr.next() == Some(pt)).unwrap();
            // ^ Unwrapping. If no square pointed to this square it would either be the head or would not be pointing to this item.
            let new_head = sqr_iter.next().unwrap();
            // ^ Unwrapping because it must exist since the previous item had a next specified in order to return.
            prev_sqr.set_next(new_head.next());
            new_head.set_next(old_head);
            *self.entries.get_mut(&item_key).unwrap() = pt;
            // ^ Unwrapping: Must trust all item_keys in a square. In the future, we might try branding the item_keys.
            true
        } else {
            false
        }
    }

    /// Adds a new entries to the EntityGrid. Takes the point in the grid to add the item to, and the
    /// Item to be added.
    ///
    /// Returns item key if successful
    /// 
    /// TODO what happens if you put the same entity twice?
    pub fn put_item(&mut self, pt: UVec2, item: Entity) -> Option<Entity> {
        if let Some(square) = self.square_mut(pt) {
            if square.item == None {
                square.item = Some(item);
                self.entries.insert(item, pt);
                Some(item)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Replaces the item for key, if one already exists
    ///
    /// Returns the previous value
    pub fn replace_item(&mut self, key: Entity, item: Entity) {
        for mut sqr in self.square_iter_mut(key) {
            sqr.set_item_key(item)
        }
    }

    /// Adds many entries to the EntityGrid. Takes an iterable of tuples of (Entity, UVec2s) where UVec2s
    /// is an iterable of [`UVec2`].
    ///
    /// The first item in the iterable of points will be the head, with the rest following in
    /// order.
    ///
    /// If any square is, closed, occupied, or out of bounds, and we try and add an item to it, that item is not
    /// added to the EntityGrid on any of the squares. Other entries will still be added though, as long
    /// as they are themselves valid.
    ///
    /// Return a Vec with the Entities of successful additions. These should be in the same order
    /// as the iterator passed to `put_entries`. If the item was not added successfully, there will
    /// be a [`None`] in its spot.
    pub fn put_entries<P: IntoIterator<Item = UVec2>, I: IntoIterator<Item = (Entity, P)>>(
        &mut self,
        entries_with_points: I,
    ) -> Vec<Option<Entity>> {
        entries_with_points
            .into_iter()
            .map(|(item, pts)| {
                let pt_vec: Vec<_> = pts.into_iter().collect();
                if pt_vec.iter().all(|pt| self.square_is_free(*pt)) {
                    let mut pt_iter = pt_vec.into_iter();
                    if let Some(head) = pt_iter.next() {
                        let key_opt = self.put_item(head, item);
                        if let Some(key) = key_opt {
                            for pt in pt_iter {
                                self.push_back(pt, key);
                            }
                        }
                        key_opt
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns a copy of the square at a certain point, or None if square is closed
    pub fn square(&self, UVec2 {x, y}: UVec2) -> Option<Square> {
        *self.grid.get(x as usize)?.get(y as usize)?
    }

    /// Square is either closed or has an item already (cannot be assigned an item)
    pub fn square_is_blocked(&self, pt: UVec2) -> bool {
        self.square_check(pt, true, Some(false))
    }
    /// Square is closed and cannot hold an item
    pub fn square_is_closed(&self, pt: UVec2) -> bool {
        self.square_check(pt, true, None)
    }

    /// Square does not contain an item (it might be closed).
    pub fn square_is_empty(&self, pt: UVec2) -> bool {
        self.square_check(pt, true, Some(true))
    }

    /// Square is open and does not contain an item
    pub fn square_is_free(&self, pt: UVec2) -> bool {
        self.square_check(pt, false, Some(true))
    }

    /// Square is open and contains an item
    pub fn square_is_occupied(&self, pt: UVec2) -> bool {
        self.square_check(pt, false, Some(false))
    }

    /// Square can contain an item (It might already contain an item)
    pub fn square_is_open(&self, pt: UVec2) -> bool {
        self.square_check(pt, false, None)
    }

    /// Iterates through all the squares that contain the item referred to by the key, from front to back.
    pub fn square_iter(&self, item_key: Entity) -> SquareIter<'_> {
        SquareIter {
            map: self,
            next: self.entries.get(&item_key).copied(),
        }
    }

    /// Returns a reference to the square at a certain point.
    ///
    /// Returns None if the point is out of bounds or closed.
    pub fn square_ref(&self, UVec2 {x, y}: UVec2) -> Option<&Square> {
        self.grid.get(x as usize)?.get(y as usize)?.as_ref()
    }

    /// Removes an item from the [`EntityGrid`], frees all squares it occupies, and returns it.
    ///
    /// Returns None if the item_key isn't valid.
    pub fn take_item(&mut self, item_key: Entity) -> Option<Entity> {
        for sqr in self.square_iter_mut(item_key) {
            sqr.clear();
        }
        self.entries.remove(&item_key).map(|_| item_key)
    }
    // HELPER FUNCTIONS

    /// Used internally for the [`square_is_X`](Self::square_is_blocked) predicates. Used to ensure that if a point is
    /// out of bounds, all the predicates will return false.
    fn square_check(&self, UVec2 {x, y}: UVec2, if_closed: bool, if_free_opt: Option<bool>) -> bool {
        match self.grid.get(x as usize).and_then(|col| col.get(y as usize)) {
            Some(sqr_opt) => match if_free_opt {
                // Square is in bounds
                None => sqr_opt.is_some() != if_closed, // We don't care about occupied status, just if it is closed or not
                Some(if_free) => match sqr_opt {
                    Some(Square { item, .. }) => item.is_some() != if_free, //Square is open, return if_free if item is free, else reverse it
                    None => if_closed,                                      // square is closed
                },
            },
            None => false, // This is out of bounds
        }
    }

    /// Allows us to iterate over the squares in a mutable way
    ///
    /// Not made public since we don't want squares to mutably accessible outside of grid map
    /// to avoid invalid states.
    fn square_iter_mut(&mut self, item_key: Entity) -> SquareIterMut<'_> {
        SquareIterMut {
            next: self.entries.get(&item_key).copied(),
            map: self, 
        }
    }

    /// Returns a mutable reference to the square at a certain point.
    ///
    /// Not made public since we don't want squares to mutably accessible outside of grid map
    /// to avoid invalid states.
    fn square_mut(&mut self, UVec2{x, y}: UVec2) -> Option<&mut Square> {
        self.grid.get_mut(x as usize)?.get_mut(y as usize)?.as_mut()
    }

    /// Returns the width of the map
    pub fn width(&self) -> u32 {
        self.width
    }
}

impl From<Vec<Vec<bool>>> for EntityGrid {
    /// Creates a [`EntityGrid`] from a collection of [`bool`], representing whether the square
    /// is open or not.
    ///
    /// Each internal [`Vec<bool>`] represents a column so that `bit_map` can be
    /// indexed like `bit_map[x][y]`.
    ///
    /// The height of the [`EntityGrid`] is determined by the maximum length of the internal
    /// [`Vec<bool>`], with others being padded at the end with closed squares. The length
    /// of the outer [`Vec<Vec<bool>>`] will determine the width.
    fn from(bit_map: Vec<Vec<bool>>) -> Self {
        let height = bit_map.iter().map(|col| col.len()).max().unwrap_or(0) as u32;
        let width = bit_map.len() as u32;

        let grid: Vec<_> = bit_map
            .iter()
            .enumerate()
            .map(|(x, col)| {
                let mut col_vec: Vec<_> = col
                    .iter()
                    .enumerate()
                    .map(|(y, sqr_key)| match sqr_key {
                        false => None,
                        true => Some(Square::new((x as u32, y as u32).into())),
                    })
                    .collect();
                col_vec.resize(height as usize, None);
                col_vec
            })
            .collect();

        EntityGrid {
            height,
            entries: HashMap::new(),
            grid,
            width,
        }
    }
}

impl<'a> SquareIterMut<'a> {
    /// Convenience method for internal functions, allows us to reverse the iterator even though it
    /// doesn't really work as a Double-Ended iterator.
    #[allow(clippy::needless_collect)]
    fn rev(self) -> Rev<IntoIter<&'a mut Square>> {
        let v: Vec<_> = self.collect();
        v.into_iter().rev()
    }
}

impl<'a> Iterator for SquareIter<'a> {
    type Item = Square;

    /// Advances the iterator and returns the next value.
    fn next(&mut self) -> Option<Self::Item> {
        self.next.and_then(|pt| self.map.square(pt)).map(|sqr| {
            self.next = sqr.next;
            sqr
        })
    }
}

impl<'a> Iterator for SquareIterMut<'a> {
    type Item = &'a mut Square;

    /// Advances the iterator and returns the next value.
    fn next(&mut self) -> Option<&'a mut Square> {
        if let Some(pt) = self.next {
            let square: &mut Square = self.map.square_mut(pt).unwrap();
            let square_ptr: *mut Square = square;

            self.next = square.next();

            // Since we have a mutable reference to map, it is unchanging while this iterator exists
            // So as long as this iterator exists, we are the only ones able to mutate squares
            // So as long as I don't allow squares to have next-pointer loops (Which would break A LOT of things),
            // this should be safe to return immutable references to separate squares.
            unsafe { square_ptr.as_mut() }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    const TEST_VALUE: &str = "TEST_VALUE";

    fn open_vertical_map<T>(n: Entity) -> EntityGrid<T> {
        EntityGrid::from(vec![vec![true; n]; 1])
    }

    fn assert_square_eq<T>(map: &EntityGrid<T>, pt: UVec2, item: Option<Entity>, next: Option<UVec2>) {
        if let Some(sqr) = map.square_ref(pt) {
            assert_eq!(item, sqr.item_key());
            assert_eq!(next, sqr.next());
        } else {
            panic!("Missing a square");
        }
    }

    fn assert_head<T>(map: &EntityGrid<T>, item_key: Entity, pt: UVec2) {
        assert_eq!(pt, map.head(item_key).unwrap());
    }

    #[test]
    fn square_iter_mut() {
        let mut map = open_vertical_map(2);
        let key = map
            .put_item((0, 0), TEST_VALUE)
            .expect("Something went wrong creating the key");
        map.push_back((0, 1), key);
        // Not something you would actually do
        let replace_key = Some(10101);
        let replace_next = Some((2, 2));
        // This invalidates the grid map, which is why square_mut and iter_square_mut are not public
        for sqr in map.square_iter_mut(key) {
            sqr.set_item_key(replace_key);
            sqr.set_next(replace_next);
        }
        assert_square_eq(&map, (0, 0), replace_key, replace_next);
        assert_square_eq(&map, (0, 1), replace_key, replace_next);
    }

    #[test]
    fn square_predicates() {
        let closed_square = (0, 0);
        let free_square = (1, 0);
        let occupied_square = (2, 0);
        let out_of_bounds = (3, 0);

        let mut map = EntityGrid::new(3, 1);

        map.open_square(free_square);
        map.open_square(occupied_square);
        map.put_item(occupied_square, "test");

        assert!(!map.square_is_open(closed_square));
        assert!(map.square_is_open(free_square));
        assert!(map.square_is_open(occupied_square));

        assert!(map.square_is_closed(closed_square));
        assert!(!map.square_is_closed(free_square));
        assert!(!map.square_is_closed(occupied_square));

        assert!(!map.square_is_free(closed_square));
        assert!(map.square_is_free(free_square));
        assert!(!map.square_is_free(occupied_square));

        assert!(map.square_is_blocked(closed_square));
        assert!(!map.square_is_blocked(free_square));
        assert!(map.square_is_blocked(occupied_square));

        assert!(map.square_is_empty(closed_square));
        assert!(map.square_is_empty(free_square));
        assert!(!map.square_is_empty(occupied_square));

        assert!(!map.square_is_occupied(closed_square));
        assert!(!map.square_is_occupied(free_square));
        assert!(map.square_is_occupied(occupied_square));

        // Out of bounds is always false
        assert!(!map.square_is_open(out_of_bounds));
        assert!(!map.square_is_closed(out_of_bounds));
        assert!(!map.square_is_free(out_of_bounds));
        assert!(!map.square_is_blocked(out_of_bounds));
        assert!(!map.square_is_empty(out_of_bounds));
        assert!(!map.square_is_occupied(out_of_bounds));
    }

    #[test]
    fn put_item() {
        let mut map = open_vertical_map(1);

        let key = map.put_item((0, 0), TEST_VALUE);

        assert_ne!(
            key, None,
            "Putting item in empty, open map should not return None"
        );
        let failed_key = map.put_item((0, 1), "This should not be allowed");
        assert_eq!(
            failed_key, None,
            "Shouldn't succeed at putting an item where another item already is"
        );

        assert_head(&map, key.unwrap(), (0, 0));
        assert_square_eq(&map, (0, 0), key, None);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn item_and_item_at() {
        let mut map = open_vertical_map(1);

        let key = map
            .put_item((0, 0), TEST_VALUE)
            .expect("Issue putting item");
        assert_eq!(map.item(key), Some(&TEST_VALUE));
        assert_eq!(map.item_at((0, 0)), Some(&TEST_VALUE));
        assert_eq!(map.item(map.square((0, 0)).unwrap()), Some(&TEST_VALUE));
    }

    #[test]
    fn take_item() {
        let mut map = open_vertical_map(3);

        let key = map
            .put_item((0, 0), TEST_VALUE)
            .expect("Error putting item");
        map.push_back((0, 1), key);
        map.push_back((0, 2), key);

        let result = map.take_item(key);

        assert_square_eq(&map, (0, 0), None, None);
        assert_square_eq(&map, (0, 1), None, None);
        assert_square_eq(&map, (0, 2), None, None);
        assert_eq!(map.len(), 0);
        assert_eq!(result, Some(TEST_VALUE));
    }

    #[test]
    fn push_back() {
        let mut map = open_vertical_map(2);

        let key = map
            .put_item((0, 0), TEST_VALUE)
            .expect("Putting item in empty, open map should not return None");

        map.push_back((0, 1), key);

        let failed_key = map.put_item((0, 1), "This should not be allowed");
        assert_eq!(
            failed_key, None,
            "Shouldn't succeed at putting an item where another item already is"
        );

        assert_head(&map, key, (0, 0));
        assert_square_eq(&map, (0, 0), Some(key), Some((0, 1)));
        assert_square_eq(&map, (0, 1), Some(key), None);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn push_front() {
        let mut map = open_vertical_map(3);

        let key = map.put_item((0, 0), String::from("UVec2 A"));
        assert_ne!(
            key, None,
            "Putting item in empty, open map should not return None"
        );

        map.push_front((0, 1), key.expect("Error putting item"));

        assert_head(&map, key.unwrap(), (0, 1));
        assert_square_eq(&map, (0, 0), key, None);
        assert_square_eq(&map, (0, 1), key, Some((0, 0)));
        assert_square_eq(&map, (0, 2), None, None);
        assert_eq!(map.len(), 1);

        // Pushing an item already in it moves it back
        map.push_front((0, 2), key.unwrap());
        map.push_front((0, 0), key.unwrap());

        assert_head(&map, key.unwrap(), (0, 0));
        assert_square_eq(&map, (0, 0), key, Some((0, 2)));
        assert_square_eq(&map, (0, 1), key, None);
        assert_square_eq(&map, (0, 2), key, Some((0, 1)));
        assert_eq!(map.len(), 1);

        // Pushing the head doesn't break it
        map.push_front((0, 0), key.unwrap());

        assert_head(&map, key.unwrap(), (0, 0));
        assert_square_eq(&map, (0, 0), key, Some((0, 2)));
        assert_square_eq(&map, (0, 1), key, None);
        assert_square_eq(&map, (0, 2), key, Some((0, 1)));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn pop_front() {
        let mut map = open_vertical_map(2);

        let key = map
            .put_item((0, 0), TEST_VALUE)
            .expect("Error putting item");
        map.push_back((0, 1), key);
        map.pop_front(key);

        assert_square_eq(&map, (0, 0), None, None);
        assert_square_eq(&map, (0, 1), Some(key), None);
        assert_head(&map, key, (0, 1));
        assert_eq!(map.len(), 1);

        map.pop_front(key);

        assert_square_eq(&map, (0, 0), None, None);
        assert_square_eq(&map, (0, 1), None, None);
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn pop_back() {
        let mut map = open_vertical_map(2);

        let key = map
            .put_item((0, 0), TEST_VALUE)
            .expect("Error putting item");
        map.push_front((0, 1), key);
        let first_pop = map.pop_back(key);

        assert_square_eq(&map, (0, 0), None, None);
        assert_square_eq(&map, (0, 1), Some(key), None);
        assert_head(&map, key, (0, 1));
        assert_eq!(map.len(), 1);
        assert_eq!(first_pop, None);

        let last_pop = map.pop_back(key);

        assert_square_eq(&map, (0, 0), None, None);
        assert_square_eq(&map, (0, 1), None, None);
        assert_eq!(map.len(), 0);
        assert_eq!(last_pop, Some(TEST_VALUE));
    }

    #[test]
    fn pop_back_n_small() {
        let mut map = open_vertical_map(3);

        let key = map
            .put_item((0, 0), TEST_VALUE)
            .expect("Error putting item");
        map.push_front((0, 1), key);
        map.push_front((0, 2), key);

        let pop_result = map.pop_back_n(key, 1);

        assert_square_eq(&map, (0, 0), None, None);
        assert_square_eq(&map, (0, 1), Some(key), None);
        assert_square_eq(&map, (0, 2), Some(key), Some((0, 1)));
        assert_head(&map, key, (0, 2));
        assert_eq!(map.len(), 1);
        assert_eq!(pop_result, None);
    }

    #[test]
    fn pop_back_n_medium() {
        let mut map = open_vertical_map(3);

        let key = map
            .put_item((0, 0), TEST_VALUE)
            .expect("Error putting item");
        map.push_front((0, 1), key);
        map.push_front((0, 2), key);

        let pop_result = map.pop_back_n(key, 2);

        assert_square_eq(&map, (0, 0), None, None);
        assert_square_eq(&map, (0, 1), None, None);
        assert_square_eq(&map, (0, 2), Some(key), None);
        assert_head(&map, key, (0, 2));
        assert_eq!(map.len(), 1);
        assert_eq!(pop_result, None);
    }

    #[test]
    fn pop_back_n_all() {
        let mut map = open_vertical_map(3);

        let key = map
            .put_item((0, 0), TEST_VALUE)
            .expect("Error putting item");
        map.push_front((0, 1), key);
        map.push_front((0, 2), key);

        let pop_result = map.pop_back_n(key, 3);

        assert_square_eq(&map, (0, 0), None, None);
        assert_square_eq(&map, (0, 1), None, None);
        assert_square_eq(&map, (0, 2), None, None);
        assert_eq!(map.len(), 0);
        assert_eq!(pop_result, Some(TEST_VALUE));
    }

    #[test]
    fn pop_front_n_small() {
        let mut map = open_vertical_map(3);

        let key = map
            .put_item((0, 0), TEST_VALUE)
            .expect("Error putting item");
        map.push_back((0, 1), key);
        map.push_back((0, 2), key);

        let pop_result = map.pop_front_n(key, 1);

        assert_square_eq(&map, (0, 0), None, None);
        assert_square_eq(&map, (0, 1), Some(key), Some((0, 2)));
        assert_square_eq(&map, (0, 2), Some(key), None);
        assert_head(&map, key, (0, 1));
        assert_eq!(map.len(), 1);
        assert_eq!(pop_result, None);
    }

    #[test]
    fn pop_front_n_medium() {
        let mut map = open_vertical_map(3);

        let key = map
            .put_item((0, 0), TEST_VALUE)
            .expect("Error putting item");
        map.push_back((0, 1), key);
        map.push_back((0, 2), key);

        let pop_result = map.pop_front_n(key, 2);

        assert_square_eq(&map, (0, 0), None, None);
        assert_square_eq(&map, (0, 1), None, None);
        assert_square_eq(&map, (0, 2), Some(key), None);
        assert_head(&map, key, (0, 2));
        assert_eq!(map.len(), 1);
        assert_eq!(pop_result, None);
    }

    #[test]
    fn pop_front_n_all() {
        let mut map = open_vertical_map(3);

        let key = map
            .put_item((0, 0), TEST_VALUE)
            .expect("Error putting item");
        map.push_back((0, 1), key);
        map.push_back((0, 2), key);

        let pop_result = map.pop_front_n(key, 3);

        assert_square_eq(&map, (0, 0), None, None);
        assert_square_eq(&map, (0, 1), None, None);
        assert_square_eq(&map, (0, 2), None, None);
        assert_eq!(map.len(), 0);
        assert_eq!(pop_result, Some(TEST_VALUE));
    }

    #[test]
    fn from_bit_map() {
        let map = EntityGrid::<String>::from(vec![vec![false, true], vec![false, true]]);

        assert!(map.square_is_closed((0, 0)));
        assert!(map.square_is_closed((1, 0)));
        assert!(map.square_is_open((0, 1)));
        assert!(map.square_is_open((1, 1)));
    }

    #[test]
    fn put_entries() {
        let mut map = open_vertical_map(7);
        let test_value_1 = "Item 2";
        let test_value_2 = "Item 3";

        let keys = map.put_entries(vec![
            (TEST_VALUE, vec![(0, 0), (0, 1), (0, 2)]),
            (test_value_1, vec![(0, 6)]),
            (test_value_2, vec![(0, 5), (0, 4)]),
            ("This value is out of bounds", vec![(3, 3)]),
            ("One of these values is already taken", vec![(0, 3), (0, 0)]),
        ]);
        assert_ne!(keys[0], None);
        assert_ne!(keys[1], None);
        assert_ne!(keys[2], None);
        assert_eq!(keys[3], None);
        assert_eq!(keys[4], None);

        assert_eq!(map.item(keys[0].unwrap()), Some(&TEST_VALUE));
        assert_eq!(map.item(keys[1].unwrap()), Some(&test_value_1));
        assert_eq!(map.item(keys[2].unwrap()), Some(&test_value_2));

        assert_square_eq(&map, (0, 0), keys[0], Some((0, 1)));
        assert_square_eq(&map, (0, 1), keys[0], Some((0, 2)));
        assert_square_eq(&map, (0, 2), keys[0], None);
        assert_square_eq(&map, (0, 3), None, None);
        assert_square_eq(&map, (0, 4), keys[2], None);
        assert_square_eq(&map, (0, 5), keys[2], Some((0, 4)));
        assert_square_eq(&map, (0, 6), keys[1], None);
    }

    #[test]
    fn return_item_with_key() {
        let mut map = open_vertical_map(3);

        let key = map
            .put_item((0, 0), TEST_VALUE)
            .expect("Error putting item");
        let key2 = map.put_item((0, 1), "Item 2");

        let pop_result = map.pop_front(key);

        assert_eq!(map.len(), 1);
        assert_ne!(pop_result, None);
        assert_square_eq(&map, (0, 0), None, None);
        assert_square_eq(&map, (0, 1), key2, None);

        unsafe {
            let return_result = map.return_item_with_key(key, (0, 0), TEST_VALUE);
            assert_eq!(return_result, Some(key));
        }
        assert_eq!(map.len(), 2);
        assert_square_eq(&map, (0, 0), Some(key), None);
        assert_square_eq(&map, (0, 1), key2, None);
    }

    #[test]
    fn number_map() {
        let mut map = EntityGrid::from(vec![vec![false, true], vec![false, true]]);

        let key = map.put_item((0, 1), TEST_VALUE).unwrap();
        assert_eq!(vec![vec![0, key], vec![0, 1],], map.number_map());
    }

    #[test]
    fn grid_shape_test() {
        let expected_grid: EntityGrid<()> = EntityGrid::from(vec![
            vec![
                false, false, false, false, false, true, false, false, false, false, false,
            ],
            vec![
                false, false, false, false, true, true, true, false, false, false, false,
            ],
            vec![
                false, false, false, true, true, true, true, true, false, false, false,
            ],
            vec![
                false, false, true, true, true, true, true, true, true, false, false,
            ],
            vec![
                false, true, true, true, true, true, true, true, true, true, false,
            ],
            vec![
                true, true, true, true, true, false, true, true, true, true, true,
            ],
            vec![
                true, true, true, true, false, false, false, true, true, true, true,
            ],
            vec![
                true, true, true, true, false, false, false, true, true, true, true,
            ],
            vec![
                false, false, false, true, true, true, true, true, false, false, false,
            ],
            vec![
                false, false, false, false, false, true, false, false, false, false, false,
            ],
            vec![
                false, false, false, true, true, true, true, true, false, false, false,
            ],
            vec![
                true, true, true, true, false, false, false, true, true, true, true,
            ],
            vec![
                true, true, true, true, false, false, false, true, true, true, true,
            ],
            vec![
                true, true, true, true, true, false, true, true, true, true, true,
            ],
            vec![
                false, true, true, true, true, true, true, true, true, true, false,
            ],
            vec![
                false, false, true, true, true, true, true, true, true, false, false,
            ],
            vec![
                false, false, false, true, true, true, true, true, false, false, false,
            ],
            vec![
                false, false, false, false, true, true, true, false, false, false, false,
            ],
            vec![
                false, false, false, false, false, true, false, false, false, false, false,
            ],
        ]);
        let shape_base_64 = "EwALACCAAz7447/vP/7x+AABPh7/+O/7jz/4gAMIAA==";
        let other_shape   = "EwALACCAAz7447/vP/6x+AABPh7/+O/7jz/4gAMIAA==";
        let grid_map_from_shape = EntityGrid::from_shape_string(shape_base_64).unwrap();
        let grid_map_from_other_shape = EntityGrid::from_shape_string(other_shape).unwrap();
        assert_eq!(expected_grid, grid_map_from_shape);
        assert_eq!(shape_base_64, expected_grid.shape_string_base64());
        assert_ne!(expected_grid, grid_map_from_other_shape);
    }
}
