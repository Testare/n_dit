use charmi::CharacterMapImage;
use crossterm::style::{ContentStyle, Stylize};
use game_core::dialogue::Dialogue;
use game_core::player::{ForPlayer, Player};

use crate::layout::{CalculatedSizeTty, StyleTty};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};

#[derive(Debug)]
pub struct DialogueUiPlugin;

impl Plugin for DialogueUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RENDER_TTY_SCHEDULE,
            (
                sys_layout_dialogue_menu.in_set(RenderTtySet::PreCalculateLayout),
                sys_render_dialogue_menu.in_set(RenderTtySet::RenderLayouts),
            ),
        );
    }
}

#[derive(Component, Debug, Default)]
pub struct DialogueMenu {}

pub fn sys_layout_dialogue_menu(
    mut q_dialogue_menu: Query<
        (&ForPlayer, Ref<CalculatedSizeTty>, &mut StyleTty),
        With<DialogueMenu>,
    >,
    q_player: Query<Ref<Dialogue>, With<Player>>,
) {
    for (&ForPlayer(player_id), size, mut style) in q_dialogue_menu.iter_mut() {
        get_assert!(player_id, q_player, |dialog| {
            use taffy::prelude::*;

            if dialog.is_changed() || size.is_changed() {
                if dialog.line().is_some() {
                    if style.size != style.max_size {
                        style.size = style.max_size;
                    }
                    /*
                    // While I typically don't like commented out code,
                    // keeping this here for a little while in case I change my mind
                    // and I want dialogue to resize
                    let expected_width = if size.width() == 0 {
                        if let Dimension::Points(width) = style.max_size.width {
                            width as usize
                        } else {
                            size.width()
                        }
                    } else {
                        size.width()
                    };
                    // TODO configure margins
                    let text_width = expected_width.saturating_sub(2);
                    let wrap_result = textwrap::wrap(line.text.as_str(), text_width);
                    let next_size = taffy::prelude::Size {
                        width: points(expected_width as f32),
                        height: points((wrap_result.len() + 5) as f32),
                    };
                    if style.size != next_size {
                        style.size = next_size;
                    }
                    */
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

pub fn sys_render_dialogue_menu(
    mut q_dialogue_menu: Query<
        (&ForPlayer, &CalculatedSizeTty, &mut TerminalRendering),
        With<DialogueMenu>,
    >,
    q_player: Query<&Dialogue, With<Player>>,
) {
    for (&ForPlayer(player_id), size, mut tr) in q_dialogue_menu.iter_mut() {
        let rendering = q_player.get(player_id).ok().and_then(|dialogue| {
            let mut charmi = CharacterMapImage::new();
            let width = size.width().checked_sub(2)?; // Margin
            let line = dialogue.line().as_ref()?;
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
            for option in dialogue.options() {
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
