use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use typed_key::Key;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(from = "HashMap<String, Value>", into = "HashMap<String, Value>")]
pub struct Metadata(HashMap<String, String>);

#[derive(Clone, Debug, Error)]
pub enum MetadataErr {
    #[error("error from serde_json in metadata: {0}")]
    SerdeError(#[from] Arc<serde_json::error::Error>),
    #[error("required metadata key not found [{0}]")]
    RequiredKeyNotFound(String),
}

impl From<serde_json::error::Error> for MetadataErr {
    fn from(value: serde_json::error::Error) -> Self {
        Self::SerdeError(Arc::new(value))
    }
}

type Result<T> = std::result::Result<T, MetadataErr>;

impl Metadata {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn new() -> Self {
        Metadata::default()
    }

    pub fn get_optional<'a, T: Deserialize<'a>>(&'a self, key: Key<T>) -> Result<Option<T>> {
        if let Some(value_str) = self.0.get(key.name()) {
            Ok(Some(serde_json::from_str(value_str)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_or_default<'a, T: Deserialize<'a> + Default>(&'a self, key: Key<T>) -> Result<T> {
        self.get_optional(key).map(|opt| opt.unwrap_or_default())
    }

    pub fn get_required<'a, T: Deserialize<'a>>(&'a self, key: Key<T>) -> Result<T> {
        if let Some(value_str) = self.0.get(key.name()) {
            Ok(serde_json::from_str(value_str)?)
        } else {
            Err(MetadataErr::RequiredKeyNotFound(key.name().to_owned()))
        }
    }

    pub fn get_field_optional<'a, T: Deserialize<'a>>(&'a self, field: &str) -> Result<Option<T>> {
        if let Some(value_str) = self.0.get(field) {
            Ok(Some(serde_json::from_str(value_str)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_field_or_default<'a, T: Deserialize<'a> + Default>(
        &'a self,
        field: &str,
    ) -> Result<T> {
        if let Some(value_str) = self.0.get(field) {
            Ok(serde_json::from_str(value_str)?)
        } else {
            Ok(Default::default())
        }
    }

    pub fn get_field_required<'a, T: Deserialize<'a>>(&'a self, field: &str) -> Result<T> {
        if let Some(value_str) = self.0.get(field) {
            Ok(serde_json::from_str(value_str)?)
        } else {
            Err(MetadataErr::RequiredKeyNotFound(field.to_owned()))
        }
    }

    pub fn put<T: Serialize, B: Borrow<T>>(&mut self, key: Key<T>, data: B) -> Result<()> {
        let data_str = serde_json::to_string(data.borrow())?;
        self.0.insert(key.name().to_string(), data_str);
        Ok(())
    }

    pub fn put_field<T: Serialize, B: Borrow<T>>(&mut self, field: &str, data: B) -> Result<()> {
        let data_str = serde_json::to_string(data.borrow())?;
        self.0.insert(field.to_string(), data_str);
        Ok(())
    }

    ///
    /// # Safety
    ///
    /// If the data is not valid JSON, this will lead to serialization/deserialization errors
    pub unsafe fn put_field_directly(&mut self, field: &str, data_str: String) {
        self.0.insert(field.to_string(), data_str);
    }

    /// Puts the data if the option is Some, else it does nothing
    pub fn put_optional<T: Serialize, O: Borrow<Option<T>>>(
        &mut self,
        key: Key<T>,
        data: O,
    ) -> Result<()> {
        if let Some(data_unwrapped) = data.borrow().as_ref() {
            self.put(key, data_unwrapped)
        } else {
            Ok(())
        }
    }

    // Possible future improvement: Trait object IsEmpty, implemented for metadata, hashmap, and Vec?
    pub fn put_nonempty<T: Serialize, V: Borrow<Vec<T>>>(
        &mut self,
        key: Key<Vec<T>>,
        data: V,
    ) -> Result<()> {
        if data.borrow().is_empty() {
            Ok(())
        } else {
            self.put(key, data.borrow())
        }
    }

    pub fn aggregate<M: IntoIterator<Item = Metadata>>(metadata: M) -> Option<Self> {
        metadata.into_iter().reduce(|mut acm, effects| {
            acm.extend(effects);
            acm
        })
    }
}

impl IntoIterator for Metadata {
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;
    type Item = (String, String);
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Extend<(String, String)> for Metadata {
    fn extend<T: IntoIterator<Item = (String, String)>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl From<HashMap<String, Value>> for Metadata {
    fn from(map: HashMap<String, Value>) -> Self {
        Metadata(
            map.into_iter()
                .map(|(key, val)| (key, val.to_string()))
                .collect(),
        )
    }
}

impl From<Metadata> for HashMap<String, Value> {
    fn from(metadata: Metadata) -> Self {
        metadata
            .0
            .into_iter()
            .map(|(key, val)| {
                (
                    key,
                    serde_json::from_str(&val).expect("Metadata should not store escaped strings"),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use typed_key::{typed_key, Key};

    use super::Metadata;

    #[test]
    #[ignore = "Problems with hashmap being unsorted"]
    pub fn hows_it_looking() {
        let mut metadata = Metadata::new();
        let key: Key<usize> = typed_key!("key1");
        let mkey: Key<HashMap<String, String>> = typed_key!("m");
        let metakey: Key<Metadata> = typed_key!("inner_metadata");
        let mut m: HashMap<String, String> = HashMap::new();
        m.insert("what".to_string(), "hey".to_string());
        m.insert("whata".to_string(), "hey".to_string());
        m.insert("foo".to_string(), "bar".to_string());
        metadata.put(key, 343).unwrap();
        metadata.put(mkey, &m).unwrap();
        metadata
            .put(metakey, &{
                let mut metadata = Metadata::new();
                metadata.put(key, 143).unwrap();
                metadata
            })
            .unwrap();
        let result = serde_json::to_string(&metadata).unwrap();
        assert_eq!("{\"m\":{\"foo\":\"bar\",\"what\":\"hey\",\"whata\":\"hey\"},\"key1\":343,\"inner_metadata\":{\"key1\":143}}", result);
    }
}
