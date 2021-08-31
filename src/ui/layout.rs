use super::super::{Bounds, Direction, GameState, Piece, Point};
use super::{SuperState, Window};
use crossterm::execute;
use std::{
    cmp,
    io::{stdout, Write},
    num::NonZeroUsize,
};
use unicode_width::UnicodeWidthStr;
// TODO Layout code encapsulation
// TODO Figure out Layout vs Render module boundaries
// TODO DrawConfiguration determines NodeLayout used?
// TODO Resize events trigger Layout recalculations

#[derive(Clone, Copy)]
pub enum NodeLayout {
    Standard(StandardNodeLayout),
    FlipMenu,
}

/// Represents all layout stuff. Its fields should be the configuration for preferred layout for in-node and out-of-node
/// Eventually, we want all layout implementation details to be obscured by this one struct
#[derive(Clone, Copy)]
pub struct Layout {
    node_layout: NodeLayout,
}

impl Layout {
    pub fn new(terminal_size: Bounds) -> Self {
        Layout {
            node_layout: NodeLayout::Standard(StandardNodeLayout::new(terminal_size, None, None)),
        }
    }

    pub fn scroll_to_pt(&mut self, game_state: &GameState, pt: Point) {
        if game_state.node().is_some() {
            self.node_layout.scroll_to_node_pt(pt);
        }
    }

    pub fn render(&self, state: &SuperState) -> std::io::Result<bool> {
        if state.game.node().is_some() {
            self.node_layout.render(state)
        } else {
            unimplemented!("TODO World map not implemented")
        }
    }
}

#[derive(Clone, Copy)]
pub struct StandardNodeLayout {
    calculated_fields: Option<StandardNodeLayoutCalculatedFields>,
    max_height: Option<NonZeroUsize>,
    max_width: Option<NonZeroUsize>,
    scroll: Point,
    terminal_bounds: Bounds,
}

#[derive(Clone, Copy)]
struct StandardNodeLayoutCalculatedFields {
    height: usize,
    include_title: bool,
    map_menu_height: usize,
    map_width: usize,
    map_window: Window,
    menu_width: usize,
    width: usize,
}

impl StandardNodeLayout {
    const SQUARE_WIDTH: usize = 3;
    const SQUARE_HEIGHT: usize = 2;
    const MIN_HEIGHT: usize = 8;
    const MIN_HEIGHT_FOR_TITLE: usize = 10;
    const MIN_WIDTH: usize = 18;

    /// If the square in the node at this point isn't fully rendered, scroll until it is
    fn scroll_to_node_pt(&mut self, pt: Point) {
        if let Some(fields) = self.calculated_fields {
            let old_scroll = self.scroll;
            // Adjusting to deal with characters instead of squares
            let char_x = pt.0 * Self::SQUARE_WIDTH;
            let char_y = pt.1 * Self::SQUARE_HEIGHT;
            let (scroll_x, scroll_y) = self.scroll;
            let x = cmp::max(
                cmp::min(scroll_x, char_x),
                (char_x + Self::SQUARE_WIDTH + 1)
                    .checked_sub(fields.map_width)
                    .unwrap_or(0),
            );
            let y = cmp::max(
                cmp::min(scroll_y, char_y),
                (char_y + Self::SQUARE_HEIGHT + 1)
                    .checked_sub(fields.map_menu_height)
                    .unwrap_or(0),
            );
            self.scroll = (x, y);
            // Optimization: Do we need to calculate all fields, or just map_window?
            if old_scroll != self.scroll {
                self.calculate_fields()
            }
        }
    }

    fn get_max_width(&self) -> usize {
        self.max_width.map(|nzu| nzu.get()).unwrap_or(120) // TODO one place for defaults
    }

    fn get_max_height(&self) -> usize {
        self.max_height.map(|nzu| nzu.get()).unwrap_or(80)
    }

    fn calculate_fields(&mut self) {
        if self.terminal_bounds.width() < Self::MIN_WIDTH
            || self.terminal_bounds.height() < Self::MIN_HEIGHT
        {
            self.calculated_fields = None;
        }

        let width = cmp::min(self.terminal_bounds.width(), self.get_max_width());
        let height = cmp::min(self.terminal_bounds.height(), self.get_max_height());
        let include_title = height >= Self::MIN_HEIGHT_FOR_TITLE;
        let menu_width = 10; // Currently safe due to MIN_WIDTH TODO customizable?
        let map_width = width - menu_width - 5;
        let map_menu_height = height - if include_title { 4 } else { 2 };
        let mut map_window = Window::of(
            NonZeroUsize::new(map_width).unwrap(),
            NonZeroUsize::new(map_menu_height).unwrap(),
        );
        map_window.scroll_x = self.scroll.0;
        map_window.scroll_y = self.scroll.1;
        /*if self.scroll != (0, 0) {
            panic!("Huh? {:?}", map_window)
        }*/

        self.calculated_fields = Some(StandardNodeLayoutCalculatedFields {
            height,
            include_title,
            map_menu_height,
            map_width,
            map_window,
            menu_width,
            width,
        })
    }

    pub fn new(
        terminal_bounds: Bounds,
        max_width: Option<NonZeroUsize>,
        max_height: Option<NonZeroUsize>,
    ) -> StandardNodeLayout {
        let mut value = StandardNodeLayout {
            calculated_fields: None,
            max_height,
            max_width,
            scroll: (0, 0),
            terminal_bounds,
        };
        value.calculate_fields();
        value
    }

    pub fn new_from_crossterm() -> crossterm::Result<StandardNodeLayout> {
        let size = crossterm::terminal::size()?;
        Ok(Self::new(Bounds::from(size), None, None))
    }

    pub fn resize(&mut self, terminal_bounds: Bounds) {
        self.terminal_bounds = terminal_bounds;
        self.calculate_fields();
    }

    pub fn scroll(&mut self, dir: Direction, speed: usize) -> bool {
        if let Some(fields) = self.calculated_fields {
            self.scroll =
                dir.add_to_point(self.scroll, speed, Bounds::of(fields.width, fields.height));
            true
        } else {
            false
        }
    }

    /// ## Safety
    /// Assumes game_state.node().is_some()
    pub unsafe fn render(&self, super_state: &SuperState) -> crossterm::Result<bool> {
        execute!(stdout(), crossterm::cursor::MoveTo(0, 0))?;
        if self.calculated_fields.is_none() {
            let (available_width, available_height) = self.terminal_bounds.into();
            for i in 0..available_height {
                let blinds = if i % 2 == 0 { ">" } else { "<" };
                execute!(
                    stdout(),
                    crossterm::style::Print(blinds.repeat(available_width)),
                    crossterm::style::Print("\n".to_string()),
                    crossterm::cursor::MoveToColumn(0)
                )?;
            }
            return Ok(false);
        }
        let fields = self.calculated_fields.unwrap(); // TODO change to an if let
        let border = '\\';
        let node = super_state.game.node().unwrap();
        execute!(
            stdout(),
            crossterm::style::Print("\\".repeat(fields.width)),
            crossterm::style::Print("\n".to_string()),
            crossterm::cursor::MoveToColumn(0)
        )?;
        if fields.include_title {
            println!(
                "{border}{0:^width$.width$}{border}",
                node.name(),
                width = fields.width - 2,
                border = border
            );
            execute!(
                stdout(),
                crossterm::cursor::MoveToColumn(0),
                crossterm::style::Print("\\".repeat(fields.width)),
                crossterm::style::Print("\n".to_string()),
                crossterm::cursor::MoveToColumn(0),
            )?;
        }

        let node_rendering = super::render::render_node(&super_state, fields.map_window);
        for (map_row, menu_row) in node_rendering.iter().zip(NodeLayout::draw_menu(
            &super_state,
            fields.height,
            fields.menu_width,
        )) {
            let row_width: usize = UnicodeWidthStr::width(map_row.as_str());
            let padding_size: usize = if row_width < fields.map_width {
                1 + fields.map_width - row_width
            } else {
                1
            };
            let menu_row_width: usize = UnicodeWidthStr::width(menu_row.as_str());
            let menu_padding_size: usize = if menu_row_width < fields.menu_width {
                fields.menu_width - menu_row_width
            } else {
                0
            }; // TODO logic to truncate if menu_row is greater than menu size...
            write!(
                stdout(),
                "{0}{1}{space:menu_padding$.menu_padding$}{0} {2}{space:padding$}{0}\n",
                border,
                menu_row,
                map_row,
                space = " ",
                menu_padding = menu_padding_size,
                padding = padding_size
            )?;
            execute!(stdout(), crossterm::cursor::MoveToColumn(0))?;
        }
        execute!(stdout(), crossterm::style::Print("/".repeat(fields.width)))?;
        Ok(true)
    }
}

impl NodeLayout {
    fn scroll_to_node_pt(&mut self, pt: Point) {
        match self {
            NodeLayout::Standard(standard_node_layout) => {
                standard_node_layout.scroll_to_node_pt(pt)
            }
            _ => unimplemented!("No other layouts yet implemented"),
        }
    }
    /**
     * Returns Result(false) if something went wrong displaying the result, such as the terminal is too small
     * Returns Err(_) if something is wrong with crossterm
     * Returns Result(true) if successfully rendered
     */
    pub fn render(&self, super_state: &SuperState) -> crossterm::Result<bool> {
        match self {
            NodeLayout::Standard(standard_node_layout) => unsafe {
                standard_node_layout.render(super_state)
            },
            _ => unimplemented!("No other layouts yet implemented"),
        }
        // TODO remove other logic, include logic for FlipMenu
    }

    pub fn draw_menu(state: &SuperState, height: usize, width: usize) -> Vec<String> {
        let pt: Point = state.selected_square();
        let piece_opt = state
            .game
            .node()
            .expect("TODO What if there is no node?")
            .piece_at(pt);
        let mut base_vec = vec![String::from(""); height];
        if let Some(piece) = piece_opt {
            match piece {
                Piece::Mon(mon_val) => {
                    base_vec[2].push_str("Money");
                    base_vec[3] = "=".repeat(width);
                    base_vec[4].push('$');
                    base_vec[4].push_str(mon_val.to_string().as_str());
                }
                Piece::AccessPoint => {
                    base_vec[2].push_str("Access Pnt");
                }
                Piece::Program(sprite) => {
                    base_vec[2].push_str("Program");
                    base_vec[3] = "=".repeat(width);
                    base_vec[4].push('[');
                    base_vec[4].push_str(sprite.display());
                    base_vec[4].push(']');
                    base_vec[5].push_str(sprite.name());
                }
            };
        }
        base_vec
    }
}

impl Default for NodeLayout {
    fn default() -> Self {
        NodeLayout::Standard(StandardNodeLayout::default())
    }
}

// TODO Default trait doesn't make sense. Not really safe
impl Default for StandardNodeLayout {
    fn default() -> Self {
        StandardNodeLayout::new_from_crossterm().unwrap()
    }
}
