use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::utils::BoxedFuture;

use super::{CharacterMapImage, CharmieActor, CharmieActorDef, CharmieDef};

#[derive(Default)]
pub struct CharmiaLoader;

#[derive(Default)]
pub struct CharmiLoader;

impl AssetLoader for CharmiaLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let toml_def = std::str::from_utf8(bytes)?;
            let actor_def: CharmieActorDef = toml::from_str(toml_def)?;
            let actor = CharmieActor::from(actor_def);
            let animations = actor.animations.clone();
            for (name, animation) in animations.into_iter() {
                load_context.set_labeled_asset(name.as_str(), LoadedAsset::new(animation));
            }
            load_context.set_default_asset(LoadedAsset::new(actor));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["charmia", "charmia.toml"]
    }
}

impl AssetLoader for CharmiLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let toml_def = std::str::from_utf8(bytes)?;
            let charmi_def: CharmieDef = toml::from_str(toml_def)?;
            let charmi = CharacterMapImage::from(charmi_def);
            load_context.set_default_asset(LoadedAsset::new(charmi));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["charmi", "charmie", "charmi.toml", "charmie.toml"]
    }
}
