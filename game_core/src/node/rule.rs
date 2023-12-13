use crate::prelude::*;

#[derive(Clone, Component, Copy, Debug, Default, Eq, PartialEq, Reflect)]
#[reflect(Component)]
pub enum AccessPointLoadingRule {
    Simultaneous,
    #[default]
    Staggered,
}
