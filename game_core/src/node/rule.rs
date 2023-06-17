use crate::prelude::*;

#[derive(Clone, Component, Copy, Default, Eq, PartialEq)]
pub enum AccessPointLoadingRule {
    Simultaneous,
    #[default]
    Staggered,
}
