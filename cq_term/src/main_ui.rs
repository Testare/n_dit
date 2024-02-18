mod main_ui_op;

use game_core::op::{OpExecutor, OpExecutorPlugin, OpPlugin};
use game_core::NDitCoreSet;
pub use main_ui_op::MainUiOp;

use crate::base_ui::context_menu::ContextMenuPane;
use crate::layout::StyleTty;
use crate::prelude::*;
use crate::render::TerminalRendering;
use crate::TerminalWindow;

#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct UiOps(OpExecutor);

#[derive(Debug, Default)]
pub struct MainUiPlugin;

impl Plugin for MainUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            OpExecutorPlugin::<UiOps>::new(Update, Some(NDitCoreSet::ProcessUiOps)),
            OpPlugin::<MainUiOp>::default(),
        ))
        .add_systems(Startup, sys_startup_create_main_ui);
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct MainUi(Option<Entity>);

pub fn sys_startup_create_main_ui(
    mut terminal_window: ResMut<TerminalWindow>,
    mut commands: Commands,
) {
    let context_menu_pane = ContextMenuPane::spawn(&mut commands);
    use taffy::prelude::*;
    let main_ui_id = commands
        .spawn((
            MainUi::default(),
            StyleTty(Style {
                size: Size {
                    width: percent(1.),
                    height: percent(1.),
                },
                display: Display::Grid,
                grid_template_rows: vec![percent(1.)],
                grid_template_columns: vec![percent(1.)],
                ..default()
            }),
            Name::new("Main Ui"),
            crate::layout::LayoutRoot,
            TerminalRendering::default(),
        ))
        .add_child(context_menu_pane)
        .id();
    terminal_window.set_render_target(Some(main_ui_id));
}
