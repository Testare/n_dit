mod node;
mod card;

use bevy::prelude::{Plugin, App};
use thiserror::Error;

pub use node::{EntityGrid, Node, NodePiece, Mon};

#[derive(Debug, Error)]
pub enum NDitError {
    #[error("attempt to decode string [{encoded_string}] but encountered error [{decode_error}]")]
    DecodeError{
        encoded_string: String,
        decode_error: String,
    }

}

pub struct NDitCorePlugin;

impl Plugin for NDitCorePlugin {
    fn build(&self, app: &mut App) {
        
    }
}
