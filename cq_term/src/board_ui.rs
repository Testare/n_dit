use charmi::{CharacterMapImage, CharmieActor, CharmieAnimation};
use game_core::board::{Board, BoardPiece, BoardPosition, BoardSize};
use game_core::registry::{Reg, Registry};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::animation::AnimationPlayer;
use crate::layout::StyleTty;
use crate::prelude::*;
use crate::render::{TerminalRendering, RENDER_TTY_SCHEDULE};

#[derive(Debug, Default)]
pub struct BoardUiPlugin;

impl Plugin for BoardUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Reg::<RegSprite>::default())
            .add_systems(RENDER_TTY_SCHEDULE, (sys_render_board, sys_render_sprites))
            .add_systems(PreUpdate, sys_board_piece_lifetimes)
            .add_systems(Update, sys_default_piece_sprites);
    }
}

#[derive(Clone, Component, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BoardPieceUi(pub Entity); // track Board Piece

impl FromWorld for BoardPieceUi {
    fn from_world(world: &mut World) -> Self {
        Self(world.spawn_empty().id())
    }
}

// TODO Should fix so that BoardUi is not mutable. If you want to track another
// board, you should create a new BoardUi.
#[derive(Clone, Component, Debug, Deref, Reflect)]
#[reflect(Component)]
pub struct BoardUi(pub Entity); // track Board

impl FromWorld for BoardUi {
    fn from_world(world: &mut World) -> Self {
        Self(world.spawn_empty().id())
    }
}

#[derive(Clone, Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BoardBackground(pub Handle<CharacterMapImage>);

/// Marker component. Can be added to BoardUI components before the
/// `system_add_default_piece_rendering` to prevent adding the
/// default components.
#[derive(Clone, Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct NoDefaultRendering;

#[derive(Component, Clone, Debug)]
pub enum Sprite {
    Image {
        image: Handle<CharacterMapImage>,
    },
    Animation {
        animation: Handle<CharmieAnimation>,
    },
    Actor {
        actor: Handle<CharmieActor>,
        starting_animation: Option<String>,
    },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum AnimationType {
    Stopped, // Animation does not play, frame is manually set
    Run,     // Animations runs once and then is done
    #[default]
    Loop, // Animation loops when finished
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum RegSprite {
    Image {
        image_path: String,
    },
    Animation {
        animation_path: String,
        #[serde(default)]
        animation_type: AnimationType,
        #[serde(default)]
        timing: Option<f32>,
    },
    Actor {
        actor_path: String,
        starting_animation: Option<String>,
    },
}

impl Registry for RegSprite {
    const REGISTRY_NAME: &'static str = "term:sprites";
    type Value = Self;
}

fn sys_render_board(
    ast_charmi: Res<Assets<CharacterMapImage>>,
    mut board_uis: Query<(AsDeref<BoardBackground>, &mut TerminalRendering)>,
) {
    for (background_handle, mut tr) in board_uis.iter_mut() {
        let charmi = ast_charmi.get(background_handle);
        if let Some(charmi) = charmi.cloned() {
            tr.update_charmie(charmi);
        }
    }
}

fn sys_board_piece_lifetimes(
    mut commands: Commands,
    created_board_uis: Query<(Entity, AsDerefCopied<BoardUi>), Added<BoardUi>>,
    board_uis: Query<(Entity, AsDerefCopied<BoardUi>)>,
    created_pieces: Query<(Entity, AsDerefCopied<Parent>), Added<BoardPiece>>,
    boards: Query<Option<AsDeref<Children>>, With<Board>>,
    board_pieces: Query<
        (
            AsDerefCopied<BoardPosition>,
            AsDerefCopiedOfCopiedOrDefault<BoardSize>,
            DebugName,
        ),
        With<BoardPiece>,
    >,
    mut removed_pieces: RemovedComponents<BoardPiece>,
    board_ui_pieces: Query<(Entity, AsDerefCopied<BoardPieceUi>)>,
) {
    // STRANGE BEHAVIOR: If an entity removes the BoardPiece component and then adds it back.
    let mut new_uis: HashSet<(Entity, Entity)> = HashSet::new();
    for (board_ui_id, board_id) in created_board_uis.iter() {
        if let Some(board_pieces) = get_assert!(board_id, boards).flatten() {
            for bp_id in board_pieces.iter() {
                new_uis.insert((board_ui_id, *bp_id));
            }
        }
    }
    for (board_id, group) in created_pieces
        .iter()
        .group_by(|(_, board_id)| *board_id)
        .into_iter()
    {
        let (new_pieces, _): (Vec<_>, Vec<_>) = group.unzip();
        for (board_ui_id, board_ui_tracks) in board_uis.iter() {
            if board_ui_tracks != board_id {
                continue;
            }
            new_uis.extend(new_pieces.iter().map(|bp_id| (board_ui_id, *bp_id)));
        }
    }
    for (board_ui_id, bp_id) in new_uis.into_iter() {
        commands.entity(board_ui_id).with_children(|board_ui| {
            if let Ok((pos, size, debug_name)) = board_pieces.get(bp_id) {
                use taffy::prelude::*;
                let grid_column = Line {
                    start: line(pos.x as i16 + 1),
                    end: span(size.x as u16),
                };
                let grid_row = Line {
                    start: line(pos.y as i16 + 1),
                    end: span(size.y as u16),
                };
                board_ui.spawn((
                    BoardPieceUi(bp_id),
                    Name::new(format!("BoardPieceUi tracking {:?}", debug_name)),
                    TerminalRendering::new(vec!["STUB".to_owned()]),
                    StyleTty(Style {
                        grid_column,
                        grid_row,
                        ..default()
                    }),
                ));
            }
        });
    }
    let removed_bp_ids: HashSet<Entity> = removed_pieces.into_iter().collect();
    for (bp_ui_id, bp_id) in board_ui_pieces.into_iter() {
        if removed_bp_ids.contains(&bp_id) {
            commands.entity(bp_ui_id).despawn();
        }
    }
}

// NOTE: This needs to run in a different phase from board_piece_lifetime
// so that they can run in the same frame
fn sys_default_piece_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    reg_sprites: Res<Reg<RegSprite>>,
    board_pieces: Query<&BoardPiece>,
    board_ui_pieces_without_sprites: Query<
        (Entity, AsDerefCopied<BoardPieceUi>),
        (Without<Sprite>, Without<NoDefaultRendering>),
    >,
) {
    for (bp_ui_id, bp_id) in board_ui_pieces_without_sprites.iter() {
        get_assert!(bp_id, board_pieces, |bp| {
            let sprite = reg_sprites.get(bp.0.as_str())?;
            match sprite {
                RegSprite::Image { image_path } => {
                    commands.entity(bp_ui_id).insert(Sprite::Image {
                        image: asset_server.load(image_path), // Perhaps I should just have there be some sort of "intermediate" for while it loads
                    });
                },
                RegSprite::Animation {
                    animation_path,
                    animation_type,
                    timing,
                } => {
                    let animation = asset_server.load(animation_path);
                    let mut animation_player = AnimationPlayer::default();
                    animation_player.load(animation.clone());
                    if let Some(timing) = timing {
                        animation_player.set_timing(*timing);
                    }
                    match animation_type {
                        AnimationType::Run => animation_player.play_once(),
                        AnimationType::Loop => animation_player.play_loop(),
                        AnimationType::Stopped => animation_player.pause(),
                    };
                    commands
                        .entity(bp_ui_id)
                        .insert((Sprite::Animation { animation }, animation_player));
                },
                _ => {},
            }

            Some(())
        });
    }
}

fn sys_render_sprites(
    ast_charmi: Res<Assets<CharacterMapImage>>,
    mut sprites: Query<(&Sprite, &mut TerminalRendering)>,
) {
    for (sprite, mut tr) in sprites.iter_mut() {
        match sprite {
            Sprite::Image { image } => {
                if let Some(img) = ast_charmi.get(image) {
                    tr.update_charmie(img.clone());
                }
            },
            _ => {},
        }
    }
}
