use crate::prelude::*;

#[derive(Clone, Component, Copy, Debug, Default, Eq, PartialEq)]
pub enum AccessPointLoadingRule {
    Simultaneous,
    #[default]
    Staggered,
}
