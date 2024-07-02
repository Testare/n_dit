use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use bevy::utils::HashMap;
use charmi::CharmiCell;
use freeform::FreeformToml;

#[derive(Debug, Default)]
pub struct CharmiPlugin {}

impl Plugin for CharmiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharmiFunctionRegistry>();
    }
}

#[derive(Debug, Default, Resource)]
pub struct CharmiFunctionRegistry {
    cell_functions: HashMap<String, SystemId<FreeformToml, Box<dyn Fn(UVec2) -> CharmiCell>>>,
    timing_functions: HashMap<String, SystemId<FreeformToml, bool>>,
}

impl CharmiFunctionRegistry {
    pub fn get_cell_function(
        world: &mut World,
        name: &str,
        freeform: FreeformToml,
    ) -> Option<Box<dyn Fn(UVec2) -> CharmiCell>> {
        let reg = world.get_resource::<Self>()?;
        let factory_id = reg.cell_functions.get(name)?;
        world.run_system_with_input(*factory_id, freeform).ok()
    }
}

pub trait RegisterCharmiFunctions {
    fn register_cell_function<F, M>(&mut self, name: &str, function: F)
    where
        F: IntoSystem<FreeformToml, Box<dyn Fn(UVec2) -> CharmiCell>, M> + 'static;
    fn register_timing_function<F, M>(&mut self, name: &str, function: F)
    where
        F: IntoSystem<FreeformToml, bool, M> + 'static;
}

impl RegisterCharmiFunctions for App {
    fn register_cell_function<F, M>(&mut self, name: &str, function: F)
    where
        F: IntoSystem<FreeformToml, Box<dyn Fn(UVec2) -> CharmiCell>, M> + 'static,
    {
        let sys_id = self.world_mut().register_system(function);
        let mut registry = self
            .world_mut()
            .get_resource_or_insert_with(CharmiFunctionRegistry::default);
        registry.cell_functions.insert(name.to_string(), sys_id);
    }

    fn register_timing_function<F, M>(&mut self, name: &str, function: F)
    where
        F: IntoSystem<FreeformToml, bool, M> + 'static,
    {
        let sys_id = self.world_mut().register_system(function);
        let mut registry = self
            .world_mut()
            .get_resource_or_insert_with(CharmiFunctionRegistry::default);
        registry.timing_functions.insert(name.to_string(), sys_id);
    }
}

#[cfg(test)]
mod test {
    use charmi::ColorValue;
    use typed_key::{typed_key, Key};

    use super::*;

    #[derive(Resource)]
    struct TestResource(u8);

    const Z_KEY: Key<u8> = typed_key!("z");

    fn test_cell_function(
        In(freeform): In<FreeformToml>,
        res_in: Res<TestResource>,
    ) -> Box<dyn Fn(UVec2) -> CharmiCell> {
        let start = res_in.0;
        let z = freeform.get_owned_or_default(Z_KEY).unwrap();
        Box::new(move |UVec2 { x, y }| {
            let shader_val = start + (x as u8) * 3 + (y as u8) * 2 - z;
            CharmiCell {
                character: None,
                fg: None,
                bg: Some(charmi::ColorValue::Ansi(shader_val)),
            }
        })
    }

    #[test]
    pub fn register_a_cell_function() {
        let mut app = App::new();
        let resource = TestResource(10u8);

        app.insert_resource(resource)
            .register_cell_function("test", test_cell_function);

        app.add_systems(Update, |world: &mut World| {
            let mut freeform = FreeformToml::new();
            freeform.put(Z_KEY, 4).unwrap();
            let cell_function = CharmiFunctionRegistry::get_cell_function(world, "test", freeform)
                .expect("Should have successfully registered test function");
            let test_input = [
                UVec2 { x: 0, y: 0 },
                UVec2 { x: 1, y: 0 },
                UVec2 { x: 0, y: 1 },
                UVec2 { x: 1, y: 1 },
            ];
            let result_cells: Vec<CharmiCell> = test_input.into_iter().map(cell_function).collect();

            assert_eq!(
                result_cells,
                vec![
                    CharmiCell {
                        bg: Some(ColorValue::Ansi(6u8)),
                        ..default()
                    },
                    CharmiCell {
                        bg: Some(ColorValue::Ansi(9u8)),
                        ..default()
                    },
                    CharmiCell {
                        bg: Some(ColorValue::Ansi(8u8)),
                        ..default()
                    },
                    CharmiCell {
                        bg: Some(ColorValue::Ansi(11u8)),
                        ..default()
                    },
                ]
            )
        });

        app.update();
    }
}
