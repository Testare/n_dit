mod button_ui;
mod grid_ui;
mod inputs;
mod menu_ui;
mod messagebar_ui;
mod node_context_actions;
mod node_glyph;
mod node_popups;
mod node_ui_op;
mod setup;
mod titlebar_ui;

use bevy::ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use bevy::reflect::Reflect;
use game_core::card::Action;
use game_core::node::EnteringNode;
use game_core::op::OpPlugin;
use game_core::player::{ForPlayer, Player};
use game_core::registry::Reg;
use game_core::NDitCoreSet;

use self::grid_ui::GridUi;
use self::menu_ui::{
    MenuUiActions, MenuUiCardSelection, MenuUiDescription, MenuUiLabel, MenuUiStats,
};
pub use self::messagebar_ui::MessageBarUi;
pub use self::node_glyph::NodeGlyph;
pub use self::node_ui_op::NodeUiOp;
use self::titlebar_ui::TitleBarUi;
use super::layout::StyleTty;
use super::render::TerminalRendering;
use crate::main_ui::{MainUiOp, UiOps};
use crate::prelude::*;
/// Plugin for NodeUI
#[derive(Debug, Default)]
pub struct NodeUiPlugin;

impl Plugin for NodeUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((Reg::<NodeGlyph>::default(), OpPlugin::<NodeUiOp>::default()))
            .init_resource::<setup::ButtonContextActions>()
            .init_resource::<node_context_actions::NodeContextActions>()
            .add_systems(Update, (setup::create_node_ui, sys_switch_screens_on_enter))
            .add_systems(
                PreUpdate,
                (inputs::kb_ready, inputs::kb_skirm_focus).in_set(NDitCoreSet::ProcessInputs),
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

#[derive(Component, Debug, Default)]
pub struct NodeUiScreen;

#[derive(Component, Debug)]
pub struct HasNodeUi;

/// Component that tells the UI which node piece the node cursor is over
///
/// Stored on Player entity
#[derive(Component, Resource, Debug, Deref, DerefMut, PartialEq)]
pub struct SelectedNodePiece(pub Option<Entity>);

#[derive(Component, Debug, Deref, DerefMut, PartialEq)]
pub struct SelectedAction(Option<usize>);

#[derive(Component, Debug, Deref, DerefMut)]
pub struct TelegraphedAction(Option<Handle<Action>>);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct AvailableMoves(HashMap<UVec2, Option<Compass>>);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct AvailableActionTargets(HashMap<UVec2, bool>);

/// Cursor that the user controls to select pieces in the node
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct NodeCursor(pub UVec2);

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct CursorIsHidden(pub bool);

impl SelectedNodePiece {
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

pub fn sys_switch_screens_on_enter(
    mut res_ui_ops: ResMut<UiOps>,
    mut q_removed_entering_node: RemovedComponents<EnteringNode>,
    q_player: Query<(), (With<HasNodeUi>, With<Player>)>,
    q_node_ui_screen: Query<(Entity, &ForPlayer), With<NodeUiScreen>>,
) {
    for player_id in q_removed_entering_node.read() {
        if q_player.contains(player_id) {
            for (node_ui_screen_id, &ForPlayer(nui_player_id)) in q_node_ui_screen.iter() {
                if nui_player_id == player_id {
                    res_ui_ops.request(player_id, MainUiOp::SwitchScreen(node_ui_screen_id));
                }
            }
        }
    }
}
