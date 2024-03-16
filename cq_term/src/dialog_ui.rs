use bevy_yarnspinner::prelude::{DialogueRunner, OptionId};
use charmi::CharacterMapImage;
use crossterm::style::{ContentStyle, Stylize};
use game_core::common::daddy::Daddy;
use game_core::dialog::Dialog;
use game_core::player::{ForPlayer, Player};
use getset::CopyGetters;
use taffy::geometry::Size;
use taffy::style::Dimension;
use taffy::style_helpers::TaffyZero;

use crate::base_ui::context_menu::ContextAction;
use crate::base_ui::HoverPoint;
use crate::layout::{CalculatedSizeTty, StyleTty};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};

#[derive(Debug, Reflect)]
pub struct DialogUiPlugin;

impl Plugin for DialogUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Daddy<DialogUiPlugin>>()
            .init_resource::<DialogUiContextActions>()
            .add_systems(
                RENDER_TTY_SCHEDULE,
                (
                    (sys_layout_dialog_line_ui, sys_layout_dialog_option_ui)
                        .in_set(RenderTtySet::PreCalculateLayout),
                    (sys_render_dialog_line_ui, sys_render_dialog_option_ui)
                        .in_set(RenderTtySet::RenderLayouts),
                ),
            );
    }
}

#[derive(CopyGetters, Debug, Resource, Reflect)]
pub struct DialogUiContextActions {
    #[getset(get_copy = "pub")]
    say_this: Entity,
}

impl FromWorld for DialogUiContextActions {
    fn from_world(world: &mut World) -> Self {
        let say_this = world
            .spawn((
                Name::new("Say this CA"),
                ContextAction::new("Say this", |id, world| {
                    (|| {
                        // try
                        let &DialogOptionUi(opt_index) = world.get(id)?;
                        let &ForPlayer(player_id) = world.get(id)?;
                        let dialog: &Dialog = world.get(player_id)?;
                        let option_exists = opt_index < dialog.options().len();
                        if option_exists {
                            let mut dialogue_runner = world.get_mut::<DialogueRunner>(player_id)?;
                            let result = dialogue_runner.select_option(OptionId(opt_index));
                            if let Err(err) = result {
                                log::error!("Error in context action [Say this]: {}", err);
                            }
                        } else if opt_index == 0 {
                            world
                                .get_mut::<DialogueRunner>(player_id)?
                                .continue_in_next_update();
                        }

                        Some(())
                    })();
                }),
            ))
            .id();

        DialogUiContextActions { say_this }
    }
}

const ZERO_SIZE: Size<Dimension> = <Size<Dimension> as TaffyZero>::ZERO;

#[derive(Component, Debug, Default)]
pub struct DialogLineUi {}

#[derive(Component, Debug, Default)]
pub struct DialogOptionUi(pub usize);

pub fn sys_layout_dialog_line_ui(
    mut q_dialog_ui: Query<(&ForPlayer, Ref<CalculatedSizeTty>, &mut StyleTty), With<DialogLineUi>>,
    q_player: Query<Ref<Dialog>, With<Player>>,
) {
    for (&ForPlayer(player_id), size, mut style) in q_dialog_ui.iter_mut() {
        get_assert!(player_id, q_player, |dialog| {
            if dialog.is_changed() || size.is_changed() {
                if dialog.line().is_some() {
                    if style.size != style.max_size {
                        style.size = style.max_size;
                    }
                } else if style.size != ZERO_SIZE {
                    style.size = ZERO_SIZE;
                }
            }
            Some(())
        });
    }
}

pub fn sys_layout_dialog_option_ui(
    mut q_dialog_ui: Query<(
        &ForPlayer,
        &DialogOptionUi,
        Ref<CalculatedSizeTty>,
        &mut StyleTty,
    )>,
    q_player: Query<Ref<Dialog>, With<Player>>,
) {
    for (&ForPlayer(player_id), &DialogOptionUi(opt_index), size, mut style) in
        q_dialog_ui.iter_mut()
    {
        get_assert!(player_id, q_player, |dialog| {
            use taffy::prelude::*;

            if dialog.is_changed() || size.is_changed() {
                if opt_index < dialog.options().len() {
                    let target_width = if size.width() < 2 {
                        if let Dimension::Length(pts) = style.max_size.width {
                            pts as usize
                        } else {
                            log::error!(
                                "Cannot configure dialog options with non-points dimensions"
                            );
                            return None;
                        }
                    } else {
                        size.width()
                    };
                    let target_height = textwrap::wrap(
                        dialog.options()[opt_index].line.text.as_str(),
                        target_width - 2,
                    )
                    .len();
                    let target_size = Size {
                        height: length(target_height as f32),
                        width: length(target_width as f32),
                    };
                    if style.size != target_size {
                        style.size = target_size;
                    }
                } else if style.size != ZERO_SIZE {
                    style.size = ZERO_SIZE;
                }
            }
            Some(())
        });
    }
}

pub fn sys_render_dialog_line_ui(
    mut q_dialog_ui: Query<
        (&ForPlayer, &CalculatedSizeTty, &mut TerminalRendering),
        With<DialogLineUi>,
    >,
    q_player: Query<&Dialog, With<Player>>,
) {
    for (&ForPlayer(player_id), size, mut tr) in q_dialog_ui.iter_mut() {
        let rendering = q_player.get(player_id).ok().and_then(|dialog| {
            let mut charmi = CharacterMapImage::new();
            let width = size.width().checked_sub(2)?; // Margin
            let line = dialog.line().as_ref()?;
            if let Some(char_name) = line.character_name() {
                // TODO map character name to full name
                charmi
                    .new_row()
                    .add_text(char_name, &ContentStyle::new().cyan());
            }
            // TODO configure at game level: Use text_without_character_name
            for line_segment in textwrap::wrap(line.text.as_str(), width) {
                charmi.new_row().add_gap(1).add_plain_text(line_segment);
            }
            Some(charmi)
        });

        if let Some(rendering) = rendering {
            tr.update_charmie(rendering);
        } else {
            tr.update(default());
        }
    }
}

pub fn sys_render_dialog_option_ui(
    mut q_dialog_ui: Query<(
        &ForPlayer,
        &DialogOptionUi,
        &CalculatedSizeTty,
        &HoverPoint,
        &mut TerminalRendering,
    )>,
    q_player: Query<&Dialog, With<Player>>,
) {
    for (&ForPlayer(player_id), &DialogOptionUi(opt_index), size, hover_point, mut tr) in
        q_dialog_ui.iter_mut()
    {
        let rendering = q_player.get(player_id).ok().and_then(|dialog| {
            let mut charmi = CharacterMapImage::new();
            let width = size.width().checked_sub(2)?; // Margin
            let line = &dialog.options().get(opt_index)?.line;
            let style = if hover_point.is_some() {
                ContentStyle::new().blue()
            } else {
                ContentStyle::new().red()
            };
            for line_segment in textwrap::wrap(line.text.as_str(), width) {
                charmi.new_row().add_gap(1).add_text(line_segment, &style);
            }
            Some(charmi)
        });

        if let Some(rendering) = rendering {
            tr.update_charmie(rendering);
        } else {
            tr.update(default());
        }
    }
}
