use std::sync::OnceLock;

use charmi::CharacterMapImage;
use game_core::node::{InNode, Node, OnTeam, TeamStatus, VictoryStatus};
use game_core::player::{ForPlayer, Player};
use indoc::indoc;

use crate::layout::VisibilityTty;
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

static HELP_MSG_IMAGE: OnceLock<CharacterMapImage> = OnceLock::new();

pub fn help_msg() -> &'static CharacterMapImage {
    // Maybe use embedded assets instead?
    HELP_MSG_IMAGE.get_or_init(|| {
        CharacterMapImage::from_toml(indoc!(
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
            y = "yellow"
            "#,
        ))
        .expect("help message should be valid toml")
    })
}

pub fn sys_render_status_screen(
    mut q_status_screen: Query<
        (
            &ForPlayer,
            &mut TerminalRendering,
            AsDerefMut<VisibilityTty>,
        ),
        With<StatusScreen>,
    >,
    q_team_status: Query<&TeamStatus, With<Node>>,
    q_player: Query<(&OnTeam, &InNode), With<Player>>,
) {
    for (&ForPlayer(player_id), mut tr, mut is_visible) in q_status_screen.iter_mut() {
        (|| {
            // TODO Improvement needed for small screen size
            let (&OnTeam(team_id), &InNode(node_id)) = q_player.get(player_id).ok()?;
            let team_status = q_team_status.get(node_id).ok()?;
            let message = match team_status.get(&team_id)? {
                VictoryStatus::Loss => "               You lost...",
                VictoryStatus::PerfectVictory => "           You won FLAWLESSLY!",
                VictoryStatus::Victory => "                You won!",
                VictoryStatus::Undecided => return None,
            };
            is_visible.set_if_neq(true);
            let mut charmi = CharacterMapImage::new();
            charmi.new_row().add_plain_text(message);
            charmi
                .new_row()
                .add_plain_text("(Press \"Quit\" to return to network map)");
            tr.update_charmie(charmi);

            Some(())
        })();
    }
}
