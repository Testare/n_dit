use n_dit::{configuration::DrawConfiguration, game::{Sprite, Node}, grid_map::GridMap, ui::Window};

fn main() {
    /*
    let region_map = [
        &[0, 0, 0, 0, 0, 0, 0, 0, 0][..],
        &[0, 0, 0, 1, 1, 1, 0, 0, 0][..],
        &[0, 0, 2, 1, 4, 4, 1, 0, 0][..],
        &[0, 1, 2, 2, 2, 1, 1, 1, 0][..],
        &[0, 0, 1, 2, 2, 1, 1, 0, 0][..],
        &[0, 0, 0, 1, 2, 1, 0, 0, 0][..],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0][..],
    ];*/
    let mut node = Node::from(GridMap::from(vec![
        vec![
            false, false, false, false, false, true, false, false, false, false, false,
        ],
        vec![
            false, false, false, false, true, true, true, false, false, false, false,
        ],
        vec![
            false, false, false, true, true, true, true, true, false, false, false,
        ],
        vec![
            false, false, true, true, true, true, true, true, true, false, false,
        ],
        vec![
            false, true, true, true, true, true, true, true, true, true, false,
        ],
        vec![
            true, true, true, true, true, false, true, true, true, true, true,
        ],
        vec![
            true, true, true, true, false, false, false, true, true, true, true,
        ],
        vec![
            true, true, true, true, false, false, false, true, true, true, true,
        ],
        vec![
            false, false, false, true, true, true, true, true, false, false, false,
        ],
        vec![
            false, false, false, false, false, true, false, false, false, false, false,
        ],
        vec![
            false, false, false, true, true, true, true, true, false, false, false,
        ],
        vec![
            true, true, true, true, false, false, false, true, true, true, true,
        ],
        vec![
            true, true, true, true, false, false, false, true, true, true, true,
        ],
        vec![
            true, true, true, true, true, false, true, true, true, true, true,
        ],
        vec![
            false, true, true, true, true, true, true, true, true, true, false,
        ],
        vec![
            false, false, true, true, true, true, true, true, true, false, false,
        ],
        vec![
            false, false, false, true, true, true, true, true, false, false, false,
        ],
        vec![
            false, false, false, false, true, true, true, false, false, false, false,
        ],
        vec![
            false, false, false, false, false, true, false, false, false, false, false,
        ],
    ]));

    let guy = Sprite::new("あ");

    let guy_key = node.add_sprite((1,6), Sprite::new("あ"));
    node.move_sprite((2,6), guy_key.unwrap());
    node.move_sprite((3,6), guy_key.unwrap());
    node.move_sprite((3,7), guy_key.unwrap());

    let guy_key = node.add_sprite((4,6), Sprite::new("死"));
    node.move_sprite((5,6), guy_key.unwrap());
    node.move_sprite((5,7), guy_key.unwrap());

    let guy_key = node.add_sprite((3,3), Sprite::new("8]"));
    node.move_sprite((3,4), guy_key.unwrap());

    let guy_key = node.add_sprite((14,6), Sprite::new("<>"));

    draw('\\', &node, None);
    draw('/', &node, None);
}

fn draw(border: char, node: &Node, window: Option<Window>) {
    for row in node.draw_node(window, &DrawConfiguration::default()) {
        println!("{0} {1} {0}", border, row);
    }
}
