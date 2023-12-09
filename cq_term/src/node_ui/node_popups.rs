use charmi::CharacterMapImage;

use crate::prelude::*;

#[derive(Component, Debug)]
pub struct HelpMenu;

#[derive(Component, Debug)]
pub struct OptionsMenu;

const HELP_MSG: &str = "
          [Click help button again to close]
-> Click on the \"@@\" spots to be able to choose cards
-> When you have choosen cards, click ready to play!
-> Each card has two stats, size and speed
-> You can move a piece a number of squares equal to speed
-> Your piece grows as it moves up to its max size
-> Attack to reduce size of enemy pieces, deleting squares
   equal to damage
-> Remove all enemy pieces to win!
";

pub fn help_msg() -> CharacterMapImage {
    HELP_MSG.lines().filter(|lin| !lin.is_empty()).collect()
}
