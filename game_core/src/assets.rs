use std::collections::HashMap;
use std::sync::Arc;
use std::ops::Index;

use serde::{Deserialize, Serialize};

pub trait Asset {
    type UnnamedAsset: Serialize + for<'de> Deserialize<'de> ;
    const SUB_EXTENSION: &'static str;

    fn with_name(unnamed: Self::UnnamedAsset, name: &str) -> Self;

}

// Impl std::ops::Index and std::ops::Extend
#[derive(Clone, Debug, Serialize)]
pub struct AssetDictionary<T: Asset>(HashMap<String, Arc<T>>);

impl <T: Asset> Default for AssetDictionary<T> {
    fn default() -> Self {
        AssetDictionary(HashMap::default())
    }
}

impl <T: Asset> Index<&str> for AssetDictionary<T> {
    type Output = Arc<T>;
    fn index(&self, idx: &str) -> &Self::Output {
        &self.0[idx]
    }
}

impl <T: Asset> AssetDictionary<T> {
    pub(crate) fn extend(&mut self, other: AssetDictionary<T>) {
        self.0.extend(other.0.into_iter())
    }

    pub fn get(&self, idx: &str) -> Option<Arc<T>> {
        self.0.get(idx).cloned()
    }
}

impl <T: Asset> From<HashMap<String, Arc<T>>> for AssetDictionary<T> {
    fn from(asset_map: HashMap<String, Arc<T>>) -> Self {
        AssetDictionary(asset_map)
    }
}
