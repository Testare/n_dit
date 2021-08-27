#[derive(PartialEq, Eq)]
pub struct Sprite {
    display: String,
    name: String,
    max_size: usize,
    moved: bool,
    team: usize,
    // actions
}

impl Sprite {
    pub fn new(display: &str) -> Sprite {
        Sprite {
            display: String::from(display),
            name: String::from("George"),
            max_size: 3,
            moved: false,
            team: 0,
        }
    }
    pub fn display(&self) -> &str {
        self.display.as_ref()
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}
