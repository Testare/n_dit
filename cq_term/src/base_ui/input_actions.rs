use std::borrow::Cow;

use bevy::ecs::entity::MapEntities;
use bevy::ecs::reflect::ReflectMapEntities;

use crate::input_event::KeyEvent;
use crate::prelude::*;

pub struct InputActionPlugin;

impl Plugin for InputActionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ContextActions>()
            .register_type::<ContextActionsDisabled>()
            .register_type::<InputAction>();
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ContextActions(Vec<InputAction>);

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ContextActionsDisabled;

#[derive(Component, Default, Reflect)]
#[reflect(MapEntities)]
pub struct LoadedInputEntities(Vec<Entity>);

impl MapEntities for LoadedInputEntities {
    fn map_entities(&mut self, entity_mapper: &mut bevy::ecs::entity::EntityMapper) {
        self.0 = self
            .0
            .drain(..)
            .map(|e| entity_mapper.get_or_reserve(e))
            .collect();
    }
}

// InputActionName -> OpRequest

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct InputAction {
    name: Cow<'static, str>,
    // user friendly name vs id?
    key_input: Option<KeyEvent>,
    mouse: bool,
    priority: i32,
}

// Should LoadedInputEntities
// ContextActions that are not disabled are triggered on key presses
// ContextActions that are under hover that are mouse triggered are listed in context menu, and the
// highest priority one is left-click. Right click is usually context menu, but might be
// configurable so that when only two actions are loaded the right click is just the second action.
//
//
// There should be a way to list the keyboard-appropriate InputActions in the mouse contextmenu
//
// InputActions lead to Ops
//
// Different concerns - What happens when the action happens, what is configured as the input, and
// when the action is active/listing what actions are available.
//
// Perhaps listing what actions are available should be the burden of the game_core? But, for
// example, listing activating a curio as available depends on where the mouse is.
//
//
// Actions on buttons
