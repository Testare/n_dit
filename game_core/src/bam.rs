use std::path::Path;
use std::sync::Arc;

use bevy::asset::{AssetLoader, LoadedUntypedAsset};
use bevy::reflect::TypePath;

use crate::prelude::*;

#[derive(Debug, Default)]
pub struct BamPlugin;

impl Plugin for BamPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<BevyAssetManifest>()
            .init_asset_loader::<BevyAssetManifestLoader>();
    }
}

#[derive(Component, Debug)]
pub struct BamHandle(pub Handle<BevyAssetManifest>);

#[derive(Asset, Debug, Default, TypePath)]
pub struct BevyAssetManifest(pub Vec<Handle<LoadedUntypedAsset>>);

#[derive(Debug, Default)]
struct BevyAssetManifestLoader;

impl AssetLoader for BevyAssetManifestLoader {
    type Asset = BevyAssetManifest;
    type Error = std::io::Error;
    type Settings = ();
    async fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader<'_>,
        _: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut file_contents = String::new();
        reader.read_to_string(&mut file_contents).await?;
        let mut root_path_buf = load_context.path().to_owned();
        root_path_buf.pop();
        let root_path: Arc<Path> = root_path_buf.as_path().into();
        let asset_handles = file_contents
            .lines()
            .map(|line| {
                let mut pathbuf = root_path.to_path_buf();
                pathbuf.push(line);
                log::trace!("BAM asset: {:?}", pathbuf);
                load_context.loader().untyped().load(pathbuf)
            })
            .collect();
        Ok(BevyAssetManifest(asset_handles))
    }

    fn extensions(&self) -> &[&str] {
        &["bam", "bam.txt"]
    }
}
