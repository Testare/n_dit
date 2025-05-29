use std::error::Error;
use std::fmt::Display;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};

use crate::{CharmiActor, CharmiActorDef, CharmiDef, CharmiImage};

#[derive(Debug, Default)]
pub struct CharmiaLoader;

#[derive(Debug, Default)]
pub struct CharmiLoader;

#[derive(Debug)]
pub enum LoaderError {
    DisappointedVoldemort(toml::de::Error),
    IllegalOmelet(std::io::Error),
}

impl AssetLoader for CharmiaLoader {
    type Asset = CharmiActor;
    type Settings = ();
    type Error = LoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<CharmiActor, Self::Error> {
        let mut toml_def = String::new();
        reader.read_to_string(&mut toml_def).await?;
        let actor_def: CharmiActorDef = toml::from_str(toml_def.as_str())?;
        let actor = CharmiActor::from(actor_def);
        let animations = actor.animations.clone();
        for (name, animation) in animations.into_iter() {
            load_context.labeled_asset_scope(name, move |_| animation);
        }
        Ok(actor)
    }

    fn extensions(&self) -> &[&str] {
        &["charmia", "charmia.toml"]
    }
}

impl AssetLoader for CharmiLoader {
    type Asset = CharmiImage;
    type Settings = ();
    type Error = LoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        _: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut toml_def = String::new();
        reader.read_to_string(&mut toml_def).await?;
        let charmi_def: CharmiDef = toml::from_str(toml_def.as_str())?;
        Ok(CharmiImage::from(&charmi_def))
    }

    fn extensions(&self) -> &[&str] {
        &["charmi", "charmi", "charmi.toml", "charmi.toml"]
    }
}

impl From<std::io::Error> for LoaderError {
    fn from(value: std::io::Error) -> Self {
        Self::IllegalOmelet(value)
    }
}

impl From<toml::de::Error> for LoaderError {
    fn from(value: toml::de::Error) -> Self {
        Self::DisappointedVoldemort(value)
    }
}

impl Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DisappointedVoldemort(e) => e.fmt(f),
            Self::IllegalOmelet(e) => e.fmt(f),
        }
    }
}

impl Error for LoaderError {}
