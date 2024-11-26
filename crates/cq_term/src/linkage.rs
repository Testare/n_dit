//! This module is an experimental architecture meant to implement functionality for mixing dependencies,
//! where submodules will be the name of the combined dependencies in alphabetical order.
//! (e.g., if we have dependency foo and dependency bar, the combined with be bar_foo)
//!
//! It is possible this will create name conflicts, for example if we merge 3
//! crates bar_foo_laser, but foo_laser is a crate by itself and we mix it with
//! bar. I don't know how likely this is, and I don't have a current solution.
//!
//! The dependency might not be a crate by itself: For example, for modules within this crate we might
//! want to build in such a way that they don't use a certain dependency for now, but we create integration
//! code within this linkage module.

pub mod base_ui_game_core;
