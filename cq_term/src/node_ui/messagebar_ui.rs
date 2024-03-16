use game_core::node::{InNode, Node, OnTeam, TeamStatus, VictoryStatus};
use game_core::player::{ForPlayer, Player};
use game_core::NDitCoreSet;

use super::{NodeUi, NodeUiQItem};
use crate::key_map::NamedInput;
use crate::layout::{CalculatedSizeTty, StyleTty};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};
use crate::{KeyMap, Submap};

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct MessageBarUi(pub Vec<String>);

#[derive(Debug, Default)]
pub struct MessageBarUiPlugin;

pub fn kb_messages(
    mut ev_keys: EventReader<KeyEvent>,
    mut message_bar_ui: Query<(&mut MessageBarUi, &ForPlayer)>,
    players: Query<(Entity, &KeyMap), With<Player>>,
) {
    for KeyEvent {
        code, modifiers, ..
    } in ev_keys.read()
    {
        for (player, key_map) in players.iter() {
            if let Some(NamedInput::NextMsg) =
                key_map.named_input_for_key(Submap::Node, *code, *modifiers)
            {
                for (mut msg_bar, ForPlayer(for_player)) in message_bar_ui.iter_mut() {
                    if *for_player == player {
                        if msg_bar.len() > 0 {
                            msg_bar.0 = msg_bar.0[1..].into();
                        }
                        break;
                    }
                }
            }
        }
    }
}

pub fn style_message_bar(mut ui: Query<(&CalculatedSizeTty, &MessageBarUi, &mut StyleTty)>) {
    use taffy::prelude::*;
    for (size, ui, mut style) in ui.iter_mut() {
        let height = length(if let Some(msg) = ui.first() {
            2.0 + textwrap::wrap(msg.as_str(), size.width()).len() as f32
        } else {
            1.0
        });
        if height != style.size.height {
            style.size.height = height;
        }
    }
}

pub fn render_message_bar(
    mut ui: Query<(&MessageBarUi, &CalculatedSizeTty, &mut TerminalRendering)>,
) {
    if let Ok((msgbar, size, mut tr)) = ui.get_single_mut() {
        let mut rendered_text: Vec<String> = vec![format!("{0:─<1$}", "─Messages", size.width())];
        if let Some(msg) = msgbar.first() {
            for line in textwrap::wrap(msg.as_str(), size.width()).into_iter() {
                rendered_text.push(line.to_string());
            }
            rendered_text.push("---Enter to continue---".to_owned());
        }
        tr.update(rendered_text);
    }
}

impl Plugin for MessageBarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (kb_messages.in_set(NDitCoreSet::ProcessInputs),))
            .add_systems(
                Update,
                sys_tmp_display_victory_or_less_message.in_set(NDitCoreSet::PostProcessCommands),
            )
            .add_systems(
                RENDER_TTY_SCHEDULE,
                (
                    style_message_bar.in_set(RenderTtySet::AdjustLayoutStyle),
                    render_message_bar.in_set(RenderTtySet::PostCalculateLayout),
                ),
            );
    }
}

impl NodeUi for MessageBarUi {
    const NAME: &'static str = "Message Bar";
    type UiBundleExtras = ();
    type UiPlugin = MessageBarUiPlugin;

    fn initial_style(_: &NodeUiQItem) -> StyleTty {
        use taffy::prelude::*;
        StyleTty(Style {
            size: Size {
                width: Dimension::Auto,
                height: length(1.),
            },
            flex_shrink: 0.0,
            ..default()
        })
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {}
}

fn sys_tmp_display_victory_or_less_message(
    nodes: Query<(Entity, AsDeref<TeamStatus>), (With<Node>, Changed<TeamStatus>)>,
    players: Query<(Entity, AsDerefCopied<InNode>, AsDerefCopied<OnTeam>), With<Player>>,
    mut message_bar_ui: IndexedQuery<ForPlayer, &mut MessageBarUi>,
) {
    for (node_id, team_status) in nodes.iter() {
        // There should be a way to know what teams have lost just now or lost before
        for (player_id, in_node, team) in players.iter() {
            if in_node != node_id {
                continue;
            }
            if let Ok(mut msgbar) = message_bar_ui.get_for_mut(player_id) {
                let msg = match team_status[&team] {
                    VictoryStatus::Undecided => continue,
                    VictoryStatus::PerfectVictory => "You won FLAWLESSLY!",
                    VictoryStatus::Victory => "You won!",
                    VictoryStatus::Loss => "You lost...",
                };
                msgbar.push(msg.to_string());
            }
        }
    }
}
