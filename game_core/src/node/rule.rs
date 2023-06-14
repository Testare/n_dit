use crate::prelude::*;

#[derive(Component)]
pub enum AccessPointLoading {
    Staggered,
    Simultaneous,
}
