mod button_ui;
mod grid_ui;
mod inputs;
mod menu_ui;
mod messagebar_ui;
mod node_glyph;
mod node_popups;
mod node_ui_op;
mod setup;
mod titlebar_ui;

use bevy::ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use bevy::reflect::Reflect;
use bevy::utils::HashSet;
use game_core::card::Action;
use game_core::op::{OpExecutorPlugin, OpPlugin};
use game_core::player::ForPlayer;
use game_core::registry::Reg;
use game_core::NDitCoreSet;

use self::grid_ui::GridUi;
use self::menu_ui::{
    MenuUiActions, MenuUiCardSelection, MenuUiDescription, MenuUiLabel, MenuUiStats,
};
pub use self::messagebar_ui::MessageBarUi;
pub use self::node_glyph::NodeGlyph;
pub use self::node_ui_op::NodeUiOp;
use self::node_ui_op::UiOps;
use self::titlebar_ui::TitleBarUi;
use super::layout::StyleTty;
use super::render::TerminalRendering;
use crate::prelude::*;
use crate::TerminalFocusMode;

/// Event that tells us to show a specific Node entity
/// Should likely be replaced with a gamecore Op
#[derive(Debug, Event)]
pub struct ShowNode {
    pub player: Entity,
    pub node: Entity,
}

#[derive(Component, Debug)]
pub struct HasNodeUi;

/// Plugin for NodeUI
#[derive(Debug, Default)]
pub struct NodeUiPlugin;

/// Component that tells the UI which entity the node cursor is over
#[derive(Component, Resource, Debug, Deref, DerefMut)]
pub struct SelectedEntity(pub Option<Entity>);

#[derive(Component, Debug, Deref, DerefMut)]
pub struct SelectedAction(Option<usize>);

#[derive(Component, Debug, Deref, DerefMut)]
pub struct TelegraphedAction(Option<Handle<Action>>);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct AvailableMoves(HashSet<UVec2>);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct AvailableActionTargets(HashSet<UVec2>);

/// Cursor that the user controls to select pieces in the node
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct NodeCursor(pub UVec2);

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct CursorIsHidden(pub bool);

impl Plugin for NodeUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Reg::<NodeGlyph>::default(),
            OpExecutorPlugin::<UiOps>::new(Update, Some(NDitCoreSet::ProcessUiOps)),
            OpPlugin::<NodeUiOp>::default(),
        ))
        .add_event::<ShowNode>()
        .add_systems(OnEnter(TerminalFocusMode::Node), setup::create_node_ui)
        .add_systems(
            PreUpdate,
            (
                inputs::kb_ready,
                inputs::kb_skirm_focus,
                button_ui::mouse_button_menu,
            )
                .in_set(NDitCoreSet::ProcessInputs),
        )
        .add_systems(
            Update,
            ((
                node_ui_op::sys_adjust_selected_action,
                node_ui_op::sys_adjust_selected_entity,
                button_ui::sys_ready_button_disable,
            )
                .chain()
                .in_set(NDitCoreSet::PostProcessUiOps),),
        )
        .add_plugins((
            MenuUiCardSelection::plugin(),
            MenuUiStats::plugin(),
            MenuUiLabel::plugin(),
            MenuUiActions::plugin(),
            MenuUiDescription::plugin(),
            GridUi::plugin(),
            MessageBarUi::plugin(),
            TitleBarUi::plugin(),
        ));
    }
}

impl SelectedEntity {
    pub fn of<'a, Q: WorldQuery, R: ReadOnlyWorldQuery>(
        &self,
        query: &'a Query<Q, R>,
    ) -> Option<<<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'a>> {
        query.get(self.0?).ok()
    }

    pub fn of_mut<'a, Q: WorldQuery, R: ReadOnlyWorldQuery>(
        &self,
        query: &'a mut Query<Q, R>,
    ) -> Option<<Q as WorldQuery>::Item<'a>> {
        query.get_mut(self.0?).ok()
    }
}

#[derive(Debug, WorldQuery)]
pub struct NodeUiQ {
    grid: &'static EntityGrid,
}

pub trait NodeUi: Component + Default {
    const NAME: &'static str;
    type UiBundleExtras: Bundle;
    type UiPlugin: Plugin + Default;

    fn ui_bundle_extras() -> Self::UiBundleExtras;
    fn initial_style(node_q: &NodeUiQItem) -> StyleTty;

    fn bundle(
        player: Entity,
        node_q: &NodeUiQItem,
    ) -> (
        StyleTty,
        Name,
        ForPlayer,
        Self::UiBundleExtras,
        Self,
        TerminalRendering,
    ) {
        (
            Self::initial_style(node_q),
            Name::new(Self::NAME),
            ForPlayer(player),
            Self::ui_bundle_extras(),
            Self::default(),
            TerminalRendering::default(),
        )
    }

    fn plugin() -> Self::UiPlugin {
        Self::UiPlugin::default()
    }
}
