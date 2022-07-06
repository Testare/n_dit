mod click;
pub mod configuration;
pub mod layout;
mod node_ui;
mod render;
mod super_state;
mod tui;
mod user_input;

use std::num::NonZeroUsize;

use click::{ClickTarget, NodeCt};
pub use configuration::DrawConfiguration;
pub(self) use configuration::{DrawType, FillMethod, UiFormat};
pub use layout::Layout;
use node_ui::NodeUiState;
use super_state::UiView;
pub use super_state::{SuperState, UiAction};
pub use user_input::UserInput;

#[derive(Clone, Copy, Debug)]
pub struct Window {
    scroll_x: usize,
    scroll_y: usize,
    width: NonZeroUsize,
    height: NonZeroUsize,
}

impl Window {
    fn of(width: NonZeroUsize, height: NonZeroUsize) -> Self {
        Window {
            height,
            scroll_x: 0,
            scroll_y: 0,
            width,
        }
    }
}

impl Default for Window {
    fn default() -> Self {
        Window {
            scroll_x: 0,
            scroll_y: 0,
            width: unsafe { NonZeroUsize::new_unchecked(usize::MAX / 4) }, // Dividing by 4 so we don't worry about overflow when adding this to scroll_x
            height: unsafe { NonZeroUsize::new_unchecked(usize::MAX / 4) },
        }
    }
}
