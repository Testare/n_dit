
pub enum MessageOp {
    SysMessage {
        from: Option<Entity>,
        targets: Vec<Entity>,
    },
    Dialog {
        target: Entity,

    },
}

