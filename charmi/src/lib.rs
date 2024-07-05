mod charmi_cell;
mod charmie_actor;
mod charmie_def;
mod fixed;
mod flexible;
mod loader;
pub mod sized;

pub use charmi_cell::CharmiCell;
pub use charmie_actor::{CharmieActor, CharmieAnimation, CharmieAnimationFrame};
pub use charmie_def::{
    CharmieActorDef, CharmieAnimationDef, CharmieDef, CharmieFrameDef, ColorDef,
};
pub use fixed::CharmiStr;
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
    Rgb(u8, u8, u8),
}

impl ColorValue {
    const BLACK: Self = Self::Ansi(0);
    const DARK_RED: Self = Self::Ansi(1);
    const DARK_GREEN: Self = Self::Ansi(2);
    const DARK_YELLOW: Self = Self::Ansi(3);
    const DARK_BLUE: Self = Self::Ansi(4);
    const DARK_MAGENTA: Self = Self::Ansi(5);
    const DARK_CYAN: Self = Self::Ansi(6);
    const GREY: Self = Self::Ansi(7);
    const DARK_GREY: Self = Self::Ansi(8);
    const RED: Self = Self::Ansi(9);
    const GREEN: Self = Self::Ansi(10);
    const YELLOW: Self = Self::Ansi(11);
    const BLUE: Self = Self::Ansi(12);
    const MAGENTA: Self = Self::Ansi(13);
    const CYAN: Self = Self::Ansi(14);
    const WHITE: Self = Self::Ansi(15);
}
