use charmi::CharacterMapImage;
use crossterm::style::{ContentStyle, Stylize};
use game_core::dialog::Dialog;
use game_core::player::{ForPlayer, Player};

use crate::layout::{CalculatedSizeTty, StyleTty};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};

#[derive(Debug)]
pub struct DialogUiPlugin;

impl Plugin for DialogUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RENDER_TTY_SCHEDULE,
            (
                sys_layout_dialog_ui.in_set(RenderTtySet::PreCalculateLayout),
                sys_render_dialog_ui.in_set(RenderTtySet::RenderLayouts),
            ),
        );
    }
}

#[derive(Component, Debug, Default)]
pub struct DialogUi {}

pub fn sys_layout_dialog_ui(
    mut q_dialog_ui: Query<(&ForPlayer, Ref<CalculatedSizeTty>, &mut StyleTty), With<DialogUi>>,
    q_player: Query<Ref<Dialog>, With<Player>>,
) {
    for (&ForPlayer(player_id), size, mut style) in q_dialog_ui.iter_mut() {
        get_assert!(player_id, q_player, |dialog| {
            use taffy::prelude::*;

            if dialog.is_changed() || size.is_changed() {
                if dialog.line().is_some() {
                    if style.size != style.max_size {
                        style.size = style.max_size;
                    }
                } else {
                    let zero_size = Size {
                        width: points(0.0),
                        height: points(0.0),
                    };
                    if style.size != zero_size {
                        style.size = zero_size;
                    }
                }
            }
            Some(())
        });
    }
}

pub fn sys_render_dialog_ui(
    mut q_dialog_ui: Query<
        (&ForPlayer, &CalculatedSizeTty, &mut TerminalRendering),
        With<DialogUi>,
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
            // TODO options should probably be children entities
            for option in dialog.options() {
                for line_segment in textwrap::wrap(option.line.text.as_str(), width) {
                    charmi
                        .new_row()
                        .add_gap(1)
                        .add_text(line_segment, &ContentStyle::new().red());
                }
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
