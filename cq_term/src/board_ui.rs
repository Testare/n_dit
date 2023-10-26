use std::ops::Deref;

use bevy::ecs::system::EntityCommands;
use charmi::{CharacterMapImage, CharmieActor, CharmieAnimation};
use game_core::board::{Board, BoardPiece, BoardPosition, BoardSize};
use game_core::player::ForPlayer;
use game_core::registry::{Reg, Registry, UpdatedRegistryKey};
use game_core::NDitCoreSet;
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
            .add_systems(
                Update,
                (
                    sys_handle_sprite_registry_updates,
                    sys_default_piece_sprites,
                )
                    .in_set(NDitCoreSet::PostProcessCommands),
            );
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

/// Component indicates that if the sprite registry is updated
/// and this key changed, we should reload the sprite.
#[derive(Clone, Component, Debug, Default, Deref, Reflect)]
#[reflect(Component)]
pub struct SpriteKey(pub String);

#[derive(Component, Clone, Debug, PartialEq)]
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

/// Describes whether the animation remains in place, runs once, or loops
/// Using dog terms for easy memorization and whimsy
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum AnimationType {
    Stay, // Animation does not play, frame is manually set
    Walk, // Animations runs once and then clears
    #[default]
    RollOver, // Animation loops when finished
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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

    fn detect_change(old_value: &Self::Value, new_value: &Self::Value) -> bool {
        old_value != new_value
    }

    fn emit_change_events() -> bool {
        true
    }
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
    created_board_uis: Query<
        (
            Entity,
            AsDerefCopied<BoardUi>,
            Option<AsDerefCopied<ForPlayer>>,
        ),
        Added<BoardUi>,
    >,
    board_uis: Query<(
        Entity,
        AsDerefCopied<BoardUi>,
        Option<AsDerefCopied<ForPlayer>>,
    )>,
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
    let mut new_uis: HashSet<(Entity, Entity, Option<Entity>)> = HashSet::new();
    for (board_ui_id, board_id, for_player) in created_board_uis.iter() {
        if let Some(board_pieces) = get_assert!(board_id, boards).flatten() {
            for bp_id in board_pieces.iter() {
                new_uis.insert((board_ui_id, *bp_id, for_player));
            }
        }
    }
    for (board_id, group) in created_pieces
        .iter()
        .group_by(|(_, board_id)| *board_id)
        .into_iter()
    {
        let (new_pieces, _): (Vec<_>, Vec<_>) = group.unzip();
        for (board_ui_id, board_ui_tracks, for_player) in board_uis.iter() {
            if board_ui_tracks != board_id {
                continue;
            }
            new_uis.extend(
                new_pieces
                    .iter()
                    .map(|bp_id| (board_ui_id, *bp_id, for_player)),
            );
        }
    }
    for (board_ui_id, bp_id, for_player) in new_uis.into_iter() {
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
                let mut bp_ui = board_ui.spawn((
                    BoardPieceUi(bp_id),
                    Name::new(format!("BoardPieceUi tracking {:?}", debug_name)),
                    StyleTty(Style {
                        grid_column,
                        grid_row,
                        ..default()
                    }),
                ));
                if let Some(for_player) = for_player {
                    bp_ui.insert(ForPlayer(for_player));
                }
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
            let sprite_key = SpriteKey(bp.0.clone());
            let mut entity_commands = commands.entity(bp_ui_id);
            entity_commands.insert((TerminalRendering::default(), sprite_key));
            sprite.update(&asset_server, entity_commands, None, None);
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

fn sys_handle_sprite_registry_updates(
    mut commands: Commands,
    mut evr_reg_sprites: EventReader<UpdatedRegistryKey<RegSprite>>,
    asset_server: Res<AssetServer>,
    reg_sprites: Res<Reg<RegSprite>>,
    mut board_uis: Query<
        (
            Entity,
            AsDeref<SpriteKey>,
            &mut Sprite,
            Option<&mut AnimationPlayer>,
        ),
        (With<BoardPieceUi>, Without<NoDefaultRendering>),
    >,
) {
    let updated_keys: HashSet<_> = evr_reg_sprites.iter().map(|d| d.deref()).collect();
    if updated_keys.is_empty() {
        return;
    }
    for (ui_id, sprite_key, sprite, animation_player) in board_uis.iter_mut() {
        if !updated_keys.contains(sprite_key) {
            continue;
        }
        if let Some(reg_sprite) = reg_sprites.get(sprite_key) {
            reg_sprite.update(
                &asset_server,
                commands.entity(ui_id),
                Some(sprite),
                animation_player,
            );
        }
    }
}

impl RegSprite {
    fn update(
        &self,
        asset_server: &AssetServer,
        mut commands: EntityCommands,
        current_sprite: Option<Mut<Sprite>>,
        animation_player: Option<Mut<AnimationPlayer>>,
    ) {
        let (next_sprite, next_ap) = match self {
            Self::Image { image_path } => (
                Sprite::Image {
                    image: asset_server.load(image_path), // Perhaps I should just have there be some sort of "intermediate" for while it loads
                },
                None,
            ),
            Self::Animation {
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
                    AnimationType::Walk => animation_player.play_once(),
                    AnimationType::RollOver => animation_player.play_loop(),
                    AnimationType::Stay => animation_player.pause(),
                };
                (Sprite::Animation { animation }, Some(animation_player))
            },
            _ => todo!("Actor logic"),
        };
        if let Some(mut current_sprite) = current_sprite {
            current_sprite.set_if_neq(next_sprite);
        } else {
            commands.insert(next_sprite);
        }
        if let Some(mut ap) = animation_player {
            if let Some(next_ap) = next_ap {
                *ap = next_ap;
            } else {
                // It seems unlikely this will change often, so might as well remove AnimationPlayer
                // If not, we can change it to ap.unload()
                commands.remove::<AnimationPlayer>();
            }
        } else if let Some(next_ap) = next_ap {
            commands.insert(next_ap);
        }
    }
}
