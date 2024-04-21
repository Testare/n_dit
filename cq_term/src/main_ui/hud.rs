use charmi::CharacterMapImage;
use crossterm::style::{ContentStyle, Stylize};
use game_core::item::Wallet;
use game_core::player::{ForPlayer, Player};

use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};

#[derive(Debug)]
pub struct HudPlugin {
    mon_display: bool,
}

impl Default for HudPlugin {
    fn default() -> Self {
        HudPlugin { mon_display: true }
    }
}

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        if self.mon_display {
            app.add_systems(
                RENDER_TTY_SCHEDULE,
                sys_render_mon_display.in_set(RenderTtySet::PostCalculateLayout),
            );
        }
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct MonDisplay(u32);

// TODO slow down this system
// TODO handle when size is too small for mon
// TODO programatically handle all magnitudes
pub fn sys_render_mon_display(
    q_player: Query<&Wallet, With<Player>>,
    mut q_mon_display: Query<(&ForPlayer, AsDerefMut<MonDisplay>, &mut TerminalRendering)>,
) {
    for (&ForPlayer(player_id), mut mon_display, mut tr) in q_mon_display.iter_mut() {
        if let Ok(wallet) = q_player.get(player_id) {
            let wallet_mon = wallet.mon();
            #[allow(clippy::comparison_chain)]
            if *mon_display < wallet_mon {
                let diff = wallet_mon - *mon_display;
                if diff < 10 {
                    *mon_display = wallet_mon;
                } else if diff < 200 {
                    *mon_display += 10;
                } else {
                    *mon_display += 100;
                }
            } else if *mon_display > wallet_mon {
                let diff = *mon_display - wallet_mon;
                if diff < 10 {
                    *mon_display = wallet_mon;
                } else if diff < 200 {
                    *mon_display -= 10;
                } else {
                    *mon_display -= 100;
                }
            }

            let next_rendering = CharacterMapImage::new().with_row(|row| {
                row.with_text(
                    format!("Mon: ${}", *mon_display),
                    &ContentStyle::new().cyan(), // TODO use color_scheme
                )
            });
            tr.update_charmie(next_rendering);
        }
    }
}
