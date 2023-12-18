use std::sync::OnceLock;

use charmi::CharacterMapImage;
use indoc::indoc;

use crate::prelude::*;

#[derive(Component, Debug)]
pub struct HelpMenu;

#[derive(Component, Debug)]
pub struct OptionsMenu;

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
