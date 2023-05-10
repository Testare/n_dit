mod entity_grid;
mod prelude;
mod card;
mod node;

pub use node::*;

use bevy::prelude::{App, Plugin};
use thiserror::Error;

pub use node::{EntityGrid, Mon, Node, NodePiece};

#[derive(Debug, Error)]
pub enum NDitError {
    #[error("attempt to decode string [{encoded_string}] but encountered error [{decode_error}]")]
    DecodeError {
        encoded_string: String,
        decode_error: String,
    },
}

pub struct NDitCorePlugin;

impl Plugin for NDitCorePlugin {
    fn build(&self, app: &mut App) {

    }
}
