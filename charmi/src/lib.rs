mod charmi_cell;
mod charmie_actor;
mod charmie_def;
mod flexible;
mod fixed;
mod loader;
pub mod sized;

pub use charmi_cell::CharmiCell;
pub use charmie_actor::{CharmieActor, CharmieAnimation, CharmieAnimationFrame};
pub use charmie_def::{
    CharmieActorDef, CharmieAnimationDef, CharmieDef, CharmieFrameDef, ColorDef,
};
pub use flexible::*;
pub use loader::{CharmiLoader, CharmiaLoader};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub enum ColorSupportLevel {
    #[default]
    TrueColor,
    Ansi256,
    Basic,
    Plain,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ColorValue {
    Ansi(u8),
    Rgb(u8, u8, u8)
}
