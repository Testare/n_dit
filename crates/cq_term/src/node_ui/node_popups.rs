use std::sync::LazyLock;

use charmi::{charmi_toml, CharmiImage};
use game_core::node::{
    Claimed, InNode, Mon, Node, OnTeam, Pickup, TeamStatus, VictoryAward, VictoryStatus,
};
use game_core::player::{ForPlayer, Player};

use crate::layout::{StyleTty, VisibilityTty};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};

#[derive(Debug)]
pub struct NodePopupsPlugin;

impl Plugin for NodePopupsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RENDER_TTY_SCHEDULE,
            (sys_render_status_screen.in_set(RenderTtySet::PostCalculateLayout),),
        );
    }
}

#[derive(Component, Debug)]
pub struct HelpMenu;

#[derive(Component, Debug)]
pub struct OptionsMenu;

#[derive(Component, Debug, Default)]
pub struct StatusScreen {}

pub static HELP_MSG_CHARMI: LazyLock<CharmiImage> = LazyLock::new(|| {
    charmi_toml![
        r#"
        text = """
                    [Click help button again to close]
        -> Click on the \"@@\" spots to be able to choose cards
        -> When you have choosen cards, click ready to play!
        -> Each card has two stats, size and speed
        -> You can move a piece a number of squares equal to speed
           (Right click or use WASD to move pieces)
        -> Your piece grows as it moves up to its max size
        -> Attack to reduce size of enemy pieces, deleting squares
        equal to damage
        -> Remove all enemy pieces to win!
        """
        fg = """
                    yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy
        """
        values.gap = "+"
        [values.colors]
        y = "yellow""#
    ]
});

pub fn sys_render_status_screen(
    mut q_status_screen: Query<
        (
            &ForPlayer,
            &mut TerminalRendering,
            AsDerefMut<VisibilityTty>,
            AsDerefMut<StyleTty>,
        ),
        With<StatusScreen>,
    >,
    q_claimed_pickup: Query<(&Claimed, &Pickup)>,
    q_player: Query<(&OnTeam, &InNode), With<Player>>,
    q_team_status: Query<&TeamStatus, With<Node>>,
    q_victory_awards: Query<(&VictoryAward, &Pickup)>,
) {
    for (&ForPlayer(player_id), mut tr, mut is_visible, mut style) in q_status_screen.iter_mut() {
        (|| {
            // TODO Improvement needed for small screen size
            let (&OnTeam(team_id), &InNode(node_id)) = q_player.get(player_id).ok()?;
            let team_status = q_team_status.get(node_id).ok()?;
            let victory_status = team_status.get(&team_id)?;
            let message = match victory_status {
                VictoryStatus::Loss => "               You lost...",
                VictoryStatus::PerfectVictory => "           You won FLAWLESSLY!",
                VictoryStatus::Victory => "                You won!",
                VictoryStatus::Undecided => return None,
            };
            is_visible.set_if_neq(true);
            let mut charmi = CharmiImage::build_dynamic();
            charmi
                .fg("magenta")
                // .bold()
                .add_line(message)
                .no_fg();
            // Victory pickups
            if victory_status.is_victorious() {
                let mut item_count = 0u32;
                let mut reward_mon = 0u32;
                for (_, pickup) in q_victory_awards
                    .iter()
                    .filter(|&(&VictoryAward(v_node_id), _)| v_node_id == node_id)
                {
                    match pickup {
                        Pickup::Mon(Mon(mon_val)) => {
                            reward_mon += mon_val;
                        },
                        Pickup::Card(_) | Pickup::Item(_) => {
                            item_count += 1;
                        },
                        _ => {},
                    }
                }
                if reward_mon > 0 {
                    charmi.add_line(&format!("* Victory credits: {}", reward_mon));
                }
                match item_count {
                    0 => {},
                    1 => {
                        charmi.add_line("* Got an item for winning");
                    },
                    _ => {
                        charmi.add_line(&format!("* Got {} items for winning", item_count));
                    },
                }
            }

            // Claimed pickups
            let mut item_count = 0u32;
            let mut reward_mon = 0u32;
            for (_, pickup) in q_claimed_pickup.iter().filter(|(claimed, _)| {
                claimed.node_id() == node_id && claimed.player() == player_id
            }) {
                match pickup {
                    Pickup::Mon(Mon(mon_val)) => {
                        reward_mon += mon_val;
                    },
                    Pickup::Card(_) | Pickup::Item(_) => {
                        item_count += 1;
                    },
                    _ => {},
                }
            }
            if reward_mon > 0 {
                charmi.add_line(&format!("* Picked up credits: {}", reward_mon));
            }
            match item_count {
                0 => {},
                1 => {
                    charmi.add_line("* Picked up an item");
                },
                _ => {
                    charmi.add_line(&format!("* Picked up {} items", item_count));
                },
            }
            charmi.add_line("(Press \"Quit\" to return to network map)");
            let charmi = charmi.build();
            style.size.height = taffy::style_helpers::length(charmi.height() as f32);
            style.max_size.height = taffy::style_helpers::length(charmi.height() as f32);
            tr.update_charmi(charmi);

            Some(())
        })();
    }
}
