//! This module handles interacting with character map images (Or Charmiimages), such as methods for creation or editting.

pub mod builder;
pub mod color;
pub mod definition;
pub mod style;

pub use builder::{CharmiBuilder, CharmiBuilderSettings};
pub use color::CharmiColor;
pub use definition::{
    CharmiActorDef, CharmiAnimationDef, CharmiDef, CharmiFrameDef, ColorDef, Values,
};
