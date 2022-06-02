use crate::Point;

#[derive(Debug, Clone, Copy)]
pub enum ClickTarget {
    Node(NodeCt),
    _World,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeCt {
    Grid(Point),
    CurioActionMenu(usize),
    _TitleButton(_TitleButtonCt),
}

#[derive(Debug, Clone, Copy)]
pub enum _TitleButtonCt {
    _QuitButton,
    _HelpButton,
}

impl From<NodeCt> for ClickTarget {
    fn from(node_ct: NodeCt) -> Self {
        ClickTarget::Node(node_ct)
    }
}
