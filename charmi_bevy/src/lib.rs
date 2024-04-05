use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use bevy::utils::HashMap;
use charmi::CharmiCell;

#[derive(Debug, Default)]
pub struct CharmiPlugin {}

impl Plugin for CharmiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharmiFunctionRegistry>();
    }
}

// Later, I want to be able to pass in more free-form information to the shader functions
// from the charmi/charmia files.
// Will probably rename Metadata type from game_core to Freeform and make its own crate
type Freeform = ();

#[derive(Debug, Default, Resource)]
pub struct CharmiFunctionRegistry {
    cell_functions: HashMap<String, SystemId<Freeform, Box<dyn Fn(UVec2) -> CharmiCell>>>,
    timing_functions: HashMap<String, SystemId<Freeform, bool>>,
}

impl CharmiFunctionRegistry {
    fn get_cell_function(
        world: &mut World,
        name: &str,
        freeform: Freeform,
    ) -> Option<Box<dyn Fn(UVec2) -> CharmiCell>> {
        let reg = world.get_resource::<Self>()?;
        let factory_id = reg.cell_functions.get(name)?;
        world.run_system_with_input(*factory_id, freeform).ok()
    }
}

pub trait RegisterCharmiFunctions {
    fn register_cell_function<F, M>(&mut self, name: &str, function: F)
    where
        F: IntoSystem<Freeform, Box<dyn Fn(UVec2) -> CharmiCell>, M> + 'static;
    fn register_timing_function<F, M>(&mut self, name: &str, function: F)
    where
        F: IntoSystem<Freeform, bool, M> + 'static;
}

impl RegisterCharmiFunctions for App {
    fn register_cell_function<F, M>(&mut self, name: &str, function: F)
    where
        F: IntoSystem<Freeform, Box<dyn Fn(UVec2) -> CharmiCell>, M> + 'static,
    {
        let sys_id = self.world.register_system(function);
        let mut registry = self
            .world
            .get_resource_or_insert_with(CharmiFunctionRegistry::default);
        registry.cell_functions.insert(name.to_string(), sys_id);
    }

    fn register_timing_function<F, M>(&mut self, name: &str, function: F)
    where
        F: IntoSystem<Freeform, bool, M> + 'static,
    {
        let sys_id = self.world.register_system(function);
        let mut registry = self
            .world
            .get_resource_or_insert_with(CharmiFunctionRegistry::default);
        registry.timing_functions.insert(name.to_string(), sys_id);
    }
}

#[cfg(test)]
mod test {
    use charmi::ColorDef;

    use super::*;

    #[derive(Resource)]
    struct TestResource(u8);

    fn test_cell_function(
        In(_): In<Freeform>,
        res_in: Res<TestResource>,
    ) -> Box<dyn Fn(UVec2) -> CharmiCell> {
        let start = res_in.0;
        Box::new(move |UVec2 { x, y }| {
            let shader_val = start + (x as u8) * 3 + (y as u8) * 2;
            CharmiCell {
                character: None,
                fg: None,
                bg: Some(charmi::ColorDef::Ansi(shader_val)),
            }
        })
    }

    const FREEFORM_STUB: Freeform = ();

    #[test]
    pub fn register_a_cell_function() {
        let mut app = App::new();
        let resource = TestResource(10u8);

        app.insert_resource(resource)
            .register_cell_function("test", test_cell_function);

        app.add_systems(Update, |world: &mut World| {
            let cell_function =
                CharmiFunctionRegistry::get_cell_function(world, "test", FREEFORM_STUB)
                    .expect("Should have successfully registered test function");
            let test_input = [
                UVec2 { x: 0, y: 0 },
                UVec2 { x: 1, y: 0 },
                UVec2 { x: 0, y: 1 },
                UVec2 { x: 1, y: 1 },
            ];
            let result_cells: Vec<CharmiCell> = test_input
                .into_iter()
                .map(|coord| cell_function(coord))
                .collect();

            assert_eq!(
                result_cells,
                vec![
                    CharmiCell {
                        bg: Some(ColorDef::Ansi(10u8)),
                        ..default()
                    },
                    CharmiCell {
                        bg: Some(ColorDef::Ansi(13u8)),
                        ..default()
                    },
                    CharmiCell {
                        bg: Some(ColorDef::Ansi(12u8)),
                        ..default()
                    },
                    CharmiCell {
                        bg: Some(ColorDef::Ansi(15u8)),
                        ..default()
                    },
                ]
            )
        });

        app.update();
    }
}
