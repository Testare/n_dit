use super::super::{render, SuperState, UiAction, Window};
use super::SubLayout;
use crate::{Bounds, Direction, Point};
use crossterm::queue;
use std::{
    cmp,
    io::{stdout, Write},
    num::NonZeroUsize,
};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, Debug)]
pub enum NodeLayout {
    Standard(StandardNodeLayout),
    FlipMenu,
}

impl SubLayout for NodeLayout {
    unsafe fn action_for_char_pt(&self, state: &SuperState, pt: Point) -> Option<UiAction> {
        self.layout().action_for_char_pt(state, pt)
    }

    fn scroll_to_pt(&mut self, pt: Point) {
        self.layout_mut().scroll_to_pt(pt)
    }

    unsafe fn render(&self, state: &SuperState) -> std::io::Result<bool> {
        self.layout().render(state)
    }

    fn resize(&mut self, terminal_size: Bounds) -> bool {
        self.layout_mut().resize(terminal_size)
    }
}

impl NodeLayout {
    /**
     * Returns Result(false) if something went wrong displaying the result, such as the terminal is too small
     * Returns Err(_) if something is wrong with crossterm
     * Returns Result(true) if successfully rendered
     */

    fn layout(&self) -> &dyn SubLayout {
        match self {
            NodeLayout::Standard(standard_node_layout) => standard_node_layout,
            _ => unimplemented!("Only standard layout has been implemented"),
        }
    }

    fn layout_mut(&mut self) -> &mut dyn SubLayout {
        match self {
            NodeLayout::Standard(standard_node_layout) => standard_node_layout,
            _ => unimplemented!("Only standard layout has been implemented"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StandardNodeLayout {
    calculated_fields: Option<StandardNodeLayoutCalculatedFields>,
    max_height: Option<NonZeroUsize>,
    max_width: Option<NonZeroUsize>,
    scroll: Point,
    terminal_bounds: Bounds,
}

// TODO Figure out better way to represent when terminal is too small for layout calculations.
#[derive(Clone, Copy, Debug)]
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
            NonZeroUsize::new(map_width).unwrap(), // TODO BUG: Panicked here
            NonZeroUsize::new(map_menu_height).unwrap(), // TODO BUG: Panicked here
        );
        map_window.scroll_x = self.scroll.0;
        map_window.scroll_y = self.scroll.1;

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

    pub fn scroll(&mut self, dir: Direction, speed: usize) -> bool {
        if let Some(fields) = self.calculated_fields {
            self.scroll =
                dir.add_to_point(self.scroll, speed, Bounds::of(fields.width, fields.height));
            true
        } else {
            false
        }
    }
}

impl SubLayout for StandardNodeLayout {
    // TODO Should probably have node_ui determine UI actions for clicks as well, with help from
    // the layout.
    /// Unsafe: Only use when the state.game_state().node().is_some() is true, otherwise this
    /// will panic
    unsafe fn action_for_char_pt(&self, state: &SuperState, pt: Point) -> Option<UiAction> {
        // TODO change logic for eager layout math and adjust for scrolling
        let (available_width, available_height) = state.terminal_size();
        if available_width < Self::MIN_WIDTH || available_height < Self::MIN_HEIGHT {
            return None;
        }
        let height = cmp::min(available_height, self.get_max_height());
        let width = cmp::min(available_width, self.get_max_width());
        let include_title = height >= Self::MIN_HEIGHT_FOR_TITLE;

        let top = if include_title { 3 } else { 1 };
        let left = 13;
        if pt.0 >= left && pt.0 < width && pt.1 >= top && pt.1 < height {
            let y = (pt.1 - top) / 2;
            let x = (pt.0 - left) / 3;
            if state
                .game_state()
                .node()
                .unwrap()
                .bounds()
                .contains_pt((x, y))
            {
                Some(UiAction::set_selected_square((x, y)))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// If the square in the node at this point isn't fully rendered, scroll until it is
    fn scroll_to_pt(&mut self, pt: Point) {
        if let Some(fields) = self.calculated_fields {
            let old_scroll = self.scroll;
            // Adjusting to deal with characters instead of squares
            let char_x = pt.0 * Self::SQUARE_WIDTH;
            let char_y = pt.1 * Self::SQUARE_HEIGHT;
            let (scroll_x, scroll_y) = self.scroll;
            let x = cmp::max(
                cmp::min(scroll_x, char_x),
                (char_x + Self::SQUARE_WIDTH + 1).saturating_sub(fields.map_width),
            );
            let y = cmp::max(
                cmp::min(scroll_y, char_y),
                (char_y + Self::SQUARE_HEIGHT + 1).saturating_sub(fields.map_menu_height),
            );
            self.scroll = (x, y);
            // Optimization: Do we need to calculate all fields, or just map_window?
            if old_scroll != self.scroll {
                self.calculate_fields()
            }
        }
    }

    /// ## Safety
    ///
    /// Assumes game_state.node().is_some()
    unsafe fn render(&self, super_state: &SuperState) -> std::io::Result<bool> {
        // TODO queue + flush over execute
        let mut stdout = stdout();
        queue!(stdout, crossterm::cursor::MoveTo(0, 0))?;
        if self.calculated_fields.is_none() {
            let (available_width, available_height) = self.terminal_bounds.into();
            for i in 0..available_height {
                let blinds = if i % 2 == 0 { ">" } else { "<" };
                queue!(
                    stdout,
                    crossterm::style::Print(blinds.repeat(available_width)),
                    crossterm::style::Print("\n".to_string()),
                    crossterm::cursor::MoveToColumn(0)
                )?;
            }
            return Ok(false);
        }
        let mon = super_state.game.player_mon();
        let fields = self.calculated_fields.unwrap(); // TODO change to an if let
        let border = '\\';
        let node = super_state.game.node().unwrap();
        queue!(
            stdout,
            crossterm::style::Print("\\".repeat(fields.width)),
            crossterm::style::Print("\n".to_string()),
            crossterm::cursor::MoveToColumn(0)
        )?;
        if fields.include_title {
            let mon_str = format!("${}", mon);
            println!(
                "{border}{0:^width$.width$} [{mon}] {border}",
                node.name(),
                width = fields.width - 6 - mon_str.len(),
                mon = mon_str,
                border = border
            );
            queue!(
                stdout,
                crossterm::cursor::MoveToColumn(0),
                crossterm::style::Print("\\".repeat(fields.width)),
                crossterm::style::Print("\n".to_string()),
                crossterm::cursor::MoveToColumn(0),
            )?;
        }

        let node_rendering = render::render_node(super_state, fields.map_window);
        for (map_row, menu_row) in node_rendering.iter().zip(render::render_menu(
            super_state,
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

            queue!(
                stdout,
                crossterm::style::Print(format!(
                    "{0}{1}{space:menu_padding$.menu_padding$}{0} {2}{space:padding$}{0}\n",
                    border,
                    menu_row,
                    map_row,
                    space = " ",
                    menu_padding = menu_padding_size,
                    padding = padding_size
                )),
                crossterm::cursor::MoveToColumn(0)
            )?;
        }
        queue!(
            stdout,
            crossterm::style::Print("/".repeat(fields.width)),
            // crossterm::style::Print("/".repeat(fields.width)),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown)
        )?;
        stdout.flush()?;
        Ok(true)
    }

    fn resize(&mut self, terminal_size: Bounds) -> bool {
        self.terminal_bounds = terminal_size;
        self.calculate_fields();
        self.calculated_fields.is_some()
        // TODO adjust scrolling after resize
    }
}
