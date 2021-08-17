use crossterm::{execute, terminal};
use super::Window;
use super::super::configuration::DrawConfiguration;
use super::super::grid_map::Point;
use super::super::game::Node;
use std::num::NonZeroUsize;
use std::io::{stdout, Write};

const ONE: NonZeroUsize = unsafe { 
    NonZeroUsize::new_unchecked(1)
};

#[derive(Clone, Copy)]
enum WorldLayout {
    Standard
}

#[derive(Clone, Copy)]
pub enum NodeLayout {
    Standard(StandardNodeLayout)
}

#[derive(Clone, Copy)]
pub struct StandardNodeLayout {
    master_width: NonZeroUsize,
    master_height: NonZeroUsize,
    window: Window,
}


impl NodeLayout {

    fn get_map_width() -> usize {
        31
    }

    fn get_map_height() -> usize {
        21
    }

    fn get_menu() -> Vec<String> {
        vec![]
    }

    fn char_position_for_point((x,y): Point) -> (usize, usize) {
        (13 + 3*x, 1 + 2*y)
    }

    pub fn draw_layout(&self, node: &Node, draw_config: &DrawConfiguration) {
        let border = '\\';
        if let NodeLayout::Standard(StandardNodeLayout { window, .. }) = self {
            // write!(stdout(), "{0}          {0} {1} {0}\n", border, row);
            // execute!(stdout(), crossterm::cursor::MoveToColumn(0));
            write!(stdout(), "/////////////////////////////////////////////////////\n");
            execute!(stdout(), crossterm::cursor::MoveToColumn(0));
            write!(stdout(), "/////////////////////////////////////////////////////\n");
            execute!(stdout(), crossterm::cursor::MoveToColumn(0));
            for row in node.draw_node(Some(*window), draw_config) {
                write!(stdout(), "{0}           {0} {1} {0}\n", border, row);
                execute!(stdout(), crossterm::cursor::MoveToColumn(0));
            }
        } 
    }

}

impl Default for NodeLayout {
    fn default() -> Self {
        NodeLayout::Standard(StandardNodeLayout::default())
    }
}

impl Default for StandardNodeLayout {
    fn default() -> Self {
        let (master_width, master_height) = terminal::size().expect("Problem getting terminal size");
        let window = Window {
            width: NonZeroUsize::new((master_width - 24).into()).unwrap_or(ONE),
            height: NonZeroUsize::new((master_height - 13).into()).unwrap_or(ONE),
            scroll_x: 0,
            scroll_y: 0,
        };


        StandardNodeLayout {
            master_width: NonZeroUsize::new(master_width.into()).unwrap_or(ONE),
            master_height: NonZeroUsize::new(master_height.into()).unwrap_or(ONE),
            window
        }
    }
}
