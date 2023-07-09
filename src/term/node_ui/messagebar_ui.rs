use crossterm::event::KeyEvent;
use game_core::player::{ForPlayer, Player};
use game_core::prelude::*;
use game_core::NDitCoreSet;
use taffy::prelude::Size;
use taffy::style::Dimension;

use super::{NodeUi, NodeUiQItem};
use crate::term::key_map::NamedInput;
use crate::term::layout::{CalculatedSizeTty, StyleTty};
use crate::term::render::{RenderTtySet, UpdateRendering};
use crate::term::{KeyMap, Submap};

#[derive(Component, Debug, Deref, DerefMut, FromReflect, Reflect)]
pub struct MessageBarUi(pub Vec<String>);

#[derive(Default)]
pub struct MessageBarUiPlugin;

pub fn kb_messages(
    mut ev_keys: EventReader<KeyEvent>,
    mut message_bar_ui: Query<(&mut MessageBarUi, &ForPlayer)>,
    players: Query<(Entity, &KeyMap), With<Player>>,
) {
    for KeyEvent { code, modifiers } in ev_keys.iter() {
        for (player, key_map) in players.iter() {
            key_map
                .named_input_for_key(Submap::Node, *code, *modifiers)
                .and_then(|named_input| {
                    if matches!(named_input, NamedInput::NextMsg) {
                        for (mut msg_bar, ForPlayer(for_player)) in message_bar_ui.iter_mut() {
                            if *for_player == player {
                                if msg_bar.len() > 0 {
                                    msg_bar.0 = msg_bar.0[1..].into();
                                }
                                break;
                            }
                        }
                    }
                    Some(())
                });
        }
    }
}

pub fn style_message_bar(mut ui: Query<(&CalculatedSizeTty, &MessageBarUi, &mut StyleTty)>) {
    for (size, ui, mut style) in ui.iter_mut() {
        let height = Dimension::Points(if let Some(msg) = ui.first() {
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
    mut commands: Commands,
    ui: Query<(Entity, &MessageBarUi, &CalculatedSizeTty)>,
) {
    if let Ok((id, msgbar, size)) = ui.get_single() {
        let mut rendered_text: Vec<String> = vec![format!("{0:─<1$}", "─Messages", size.width())];
        if let Some(msg) = msgbar.first() {
            for line in textwrap::wrap(msg.as_str(), size.width()).into_iter() {
                rendered_text.push(line.to_string());
            }
            rendered_text.push("---Enter to continue---".to_owned());
        }
        commands
            .get_entity(id)
            .update_rendering(rendered_text.clone());
    }
}

impl Plugin for MessageBarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((
            kb_messages.in_set(NDitCoreSet::ProcessInputs),
            style_message_bar.in_set(RenderTtySet::PreCalculateLayout),
            render_message_bar.in_set(RenderTtySet::PostCalculateLayout),
        ));
    }
}

impl NodeUi for MessageBarUi {
    const NAME: &'static str = "Message Bar";
    type UiBundleExtras = ();
    type UiPlugin = MessageBarUiPlugin;

    fn initial_style(_: &NodeUiQItem) -> StyleTty {
        StyleTty(taffy::prelude::Style {
            size: Size {
                width: Dimension::Auto,
                height: Dimension::Points(1.),
            },
            flex_shrink: 0.0,
            ..default()
        })
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {
        ()
    }
}

impl Default for MessageBarUi {
    fn default() -> Self {
        super::MessageBarUi(vec!["Have you ever heard the story of Darth Plegius the wise? I thought not, it's not a story the jedi would tell you. He was powerful, some say he even could even stop people from dying. Of course, he was betrayed, and at this point Logan's memory starts to fail, and he isn't really able to quote the whole thing exactly. But of course I remember the gist.".to_owned()])
    }
}
