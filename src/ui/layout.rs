use super::super::{Piece, Point};
use super::{SuperState, Window};
use crossterm::execute;
use std::{
    cmp,
    io::{stdout, Write},
    num::NonZeroUsize,
};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy)]
pub enum NodeLayout {
    Standard(StandardNodeLayout),
    FlipMenu,
}

#[derive(Clone, Copy)]
pub struct StandardNodeLayout {
    max_width: Option<NonZeroUsize>,
    max_height: Option<NonZeroUsize>,
}

impl NodeLayout {
    fn get_max_width(&self) -> usize {
        match self {
            Self::Standard(StandardNodeLayout { max_width, .. }) => {
                max_width.map(|nzu| nzu.get()).unwrap_or(120)
            }
            Self::FlipMenu => unimplemented!("Not yet implemented"),
        }
    }

    fn get_max_height(&self) -> usize {
        match self {
            Self::Standard(StandardNodeLayout { max_height, .. }) => {
                max_height.map(|nzu| nzu.get()).unwrap_or(24)
            }
            Self::FlipMenu => unimplemented!("Not yet implemented"),
        }
    }

    const MIN_HEIGHT: usize = 8;
    const MIN_HEIGHT_FOR_TITLE: usize = 10;
    const MIN_WIDTH: usize = 18;

    /**
     * Returns Result(false) if something went wrong displaying the result, such as the terminal is too small
     * Returns Err(_) if something is wrong with crossterm
     * Returns Result(true) if successfully rendered
     */
    pub fn render(&self, super_state: &SuperState) -> crossterm::Result<bool> {
        execute!(stdout(), crossterm::cursor::MoveTo(0, 0))?;
        let (available_width, available_height) = super_state.terminal_size();

        if available_width < Self::MIN_WIDTH || available_height < Self::MIN_HEIGHT {
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
        let width = cmp::min(available_width, self.get_max_width());
        let height = cmp::min(available_height, self.get_max_height());
        let include_title = height >= Self::MIN_HEIGHT_FOR_TITLE;
        let border = '\\';
        let node = super_state.game.node().unwrap(); // TODO how to handle no Node
        let menu_width = 10;
        let map_width = width - menu_width - 5;
        let map_menu_height = height - if include_title { 4 } else { 2 };
        let map_window = Window::of(
            NonZeroUsize::new(map_width).unwrap(),
            NonZeroUsize::new(map_menu_height).unwrap(),
        );
        execute!(
            stdout(),
            crossterm::style::Print("\\".repeat(width)),
            crossterm::style::Print("\n".to_string()),
            crossterm::cursor::MoveToColumn(0)
        )?;
        if include_title {
            println!(
                "{border}{0:^width$.width$}{border}",
                node.name(),
                width = width - 2,
                border = border
            );
            execute!(
                stdout(),
                crossterm::cursor::MoveToColumn(0),
                crossterm::style::Print("\\".repeat(width)),
                crossterm::style::Print("\n".to_string()),
                crossterm::cursor::MoveToColumn(0),
            )?;
        }

        // for row in node.draw_node(Some(map_window), draw_config) {

        let node_rendering = super::render::render_node(&super_state, map_window);
        for (map_row, menu_row) in
            node_rendering
                .iter()
                .zip(Self::draw_menu(&super_state, height, menu_width))
        {
            let row_width: usize = UnicodeWidthStr::width(map_row.as_str());
            let padding_size: usize = if row_width < map_width {
                1 + map_width - row_width
            } else {
                1
            };
            let menu_row_width: usize = UnicodeWidthStr::width(menu_row.as_str());
            let menu_padding_size: usize = if menu_row_width < menu_width {
                menu_width - menu_row_width
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
        execute!(stdout(), crossterm::style::Print("/".repeat(width)))?;
        Ok(true)
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

impl Default for StandardNodeLayout {
    fn default() -> Self {
        StandardNodeLayout {
            max_width: None,
            max_height: None,
        }
    }
}
