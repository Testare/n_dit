use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::utils::thiserror::Error;
use bevy::utils::BoxedFuture;

use super::{CharacterMapImage, CharmieActor, CharmieActorDef, CharmieDef};

#[derive(Debug, Default)]
pub struct CharmiaLoader;

#[derive(Debug, Default)]
pub struct CharmiLoader;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error(transparent)]
    DisappointedVoldemort(#[from] toml::de::Error),
    #[error(transparent)]
    IllegalOmelet(#[from] std::io::Error),
}

impl AssetLoader for CharmiaLoader {
    type Asset = CharmieActor;
    type Settings = ();
    type Error = LoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<CharmieActor, Self::Error>> {
        Box::pin(async move {
            let mut toml_def = String::new();
            reader.read_to_string(&mut toml_def).await?;
            let actor_def: CharmieActorDef = toml::from_str(toml_def.as_str())?;
            let actor = CharmieActor::from(actor_def);
            let animations = actor.animations.clone();
            for (name, animation) in animations.into_iter() {
                load_context.labeled_asset_scope(name, move |_| animation);
            }
            Ok(actor)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["charmia", "charmia.toml"]
    }
}

impl AssetLoader for CharmiLoader {
    type Asset = CharacterMapImage;
    type Settings = ();
    type Error = LoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &Self::Settings,
        _: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut toml_def = String::new();
            reader.read_to_string(&mut toml_def).await?;
            let charmi_def: CharmieDef = toml::from_str(toml_def.as_str())?;
            Ok(CharacterMapImage::from(charmi_def))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["charmi", "charmie", "charmi.toml", "charmie.toml"]
    }
}
