use std::{
    cmp,
    io::{stdout, Write},
    num::NonZeroUsize,
};

use crossterm::queue;
use old_game_core::{Bounds, Direction, GameState, Point};
use unicode_width::UnicodeWidthStr;

use super::super::{render, ClickTarget, NodeCt, NodeUiState, SuperState, UiAction, Window};
use super::SubLayout;

#[derive(Clone, Copy, Debug)]
pub enum NodeLayout {
    Standard(StandardNodeLayout),
    FlipMenu,
}

impl SubLayout for NodeLayout {
    fn apply_action(&mut self, ui_action: &UiAction, node_ui: Option<&NodeUiState>) {
        self.layout_mut().apply_action(ui_action, node_ui)
    }

    fn scroll_to_pt(&mut self, pt: Point) {
        self.layout_mut().scroll_to_pt(pt)
    }

    unsafe fn render(&self, state: &SuperState, game_state: &GameState) -> std::io::Result<bool> {
        self.layout().render(state, game_state)
    }

    fn resize(&mut self, terminal_size: Bounds) -> bool {
        self.layout_mut().resize(terminal_size)
    }

    fn click_target(
        &self,
        state: &SuperState,
        game_state: &GameState,
        pt: Point,
    ) -> Option<ClickTarget> {
        self.layout().click_target(state, game_state, pt)
    }
}

impl NodeLayout {
    pub fn terminal_size(&self) -> Bounds {
        match self {
            NodeLayout::Standard(standard_node_layout) => standard_node_layout.terminal_bounds,
            _ => unimplemented!("Only standard layout has been implemented"),
        }
    }

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
    const MIN_HEIGHT: usize = 10;
    const MIN_HEIGHT_FOR_TITLE: usize = 12;
    const MIN_WIDTH: usize = 30;
    const CURIO_ACTION_Y: usize = 7;

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
            return;
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

        self.calculated_fields = Some(StandardNodeLayoutCalculatedFields {
            height,
            include_title,
            map_menu_height,
            map_width,
            map_window,
            menu_width,
            width,
        });
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
    fn apply_action(&mut self, ui_action: &UiAction, node_ui: Option<&NodeUiState>) {
        let scroll_to_pt = match ui_action {
            UiAction::MoveSelectedSquare { .. } => Some(
                node_ui
                    .expect("Node UI should exist if we're using Node Layout")
                    .selected_square(),
            ),
            UiAction::SetSelectedSquare(pt) => Some(*pt),
            _ => None,
        };
        if let Some(pt) = scroll_to_pt {
            self.scroll_to_pt(pt);
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

    /**
     * Returns Result(false) if something went wrong displaying the result, such as the terminal is too small
     * Returns Err(_) if something is wrong with crossterm
     * Returns Result(true) if successfully rendered
     */
    /// ## Safety
    ///
    /// Assumes game_state.node().is_some()
    unsafe fn render(
        &self,
        super_state: &SuperState,
        game_state: &GameState,
    ) -> std::io::Result<bool> {
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
        let mon = game_state.player_mon();
        let fields = self.calculated_fields.unwrap(); // TODO change to an if let
        let border = '\\';
        let node = game_state.node().unwrap();
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

        let node_rendering = render::render_node(node, super_state, fields.map_window);
        for (map_row, menu_row) in node_rendering.iter().zip(render::render_menu(
            super_state,
            game_state,
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

    fn click_target(
        &self,
        state: &SuperState,
        game_state: &GameState,
        pt: Point,
    ) -> Option<ClickTarget> {
        // TODO change logic for eager layout math
        let Bounds(available_width, available_height) = state.terminal_size();
        if available_width < Self::MIN_WIDTH || available_height < Self::MIN_HEIGHT {
            return None;
        }
        let height = cmp::min(available_height, self.get_max_height());
        let width = cmp::min(available_width, self.get_max_width());
        let include_title = height >= Self::MIN_HEIGHT_FOR_TITLE;

        let top = if include_title { 3 } else { 1 };
        let left = 13;
        if pt.1 >= top && pt.1 < height {
            if pt.0 > 0 && pt.0 < left {
                let node = game_state.node().unwrap();
                // Action Menus
                (pt.1 - top)
                    .checked_sub(Self::CURIO_ACTION_Y)
                    .and_then(|index| {
                        node.with_curio_at(state.selected_square(), |curio| curio.action_count())
                            .filter(|available_action_total| *available_action_total >= index)
                            .map(|_| NodeCt::CurioActionMenu(index).into())
                    })
            } else if pt.0 < width {
                let (sx, sy) = self.scroll;
                let y = (pt.1 + sy - top) / 2;
                let x = (pt.0 + sx - left) / 3;
                log::debug!(
                    "Scroll: {:?} / pt: {:?} / calculated grid_pt: {:?}",
                    self.scroll,
                    pt,
                    (x, y)
                );
                if game_state.node().unwrap().bounds().contains_pt((x, y)) {
                    Some(NodeCt::Grid((x, y)).into())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}
