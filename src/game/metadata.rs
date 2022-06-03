use super::error::{ErrorMsg as _, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Borrow;
use std::collections::HashMap;
use typed_key::Key;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(from = "HashMap<String, Value>", into = "HashMap<String, Value>")]
pub struct Metadata(HashMap<String, String>);

impl Metadata {
    pub fn new() -> Self {
        Metadata::default()
    }

    pub fn get<'a, T: Deserialize<'a>>(&'a self, key: Key<T>) -> Result<Option<T>> {
        self.0
            .get(&key.name().to_string())
            .map(|data| {
                serde_json::from_str(data).map_err(|e| {
                    format!(
                        "Error occured deserializing metadata key [{}], [{}]",
                        key.name(),
                        e
                    )
                    .fail_critical_msg()
                })
            })
            .transpose()
    }

    pub fn get_field<'a, T: Deserialize<'a>>(&'a self, field: &str) -> Result<Option<T>> {
        self.0
            .get(&field.to_string())
            .map(|data| {
                serde_json::from_str(data).map_err(|e| {
                    format!(
                        "Error occured deserializing metadata field [{}], [{}]",
                        field, e
                    )
                    .fail_critical_msg()
                })
            })
            .transpose()
    }

    pub fn get_or_default<'a, T: Deserialize<'a> + Default>(&'a self, key: Key<T>) -> Result<T> {
        self.get(key).map(Option::unwrap_or_default)
    }

    pub fn expect<'a, T: Deserialize<'a>>(&'a self, key: Key<T>) -> Result<T> {
        if let Some(data) = self.0.get(&key.name().to_string()) {
            serde_json::from_str(data).map_err(|e| {
                format!(
                    "Error occured deserializing expected metadata key [{}], [{}]",
                    key.name(),
                    e
                )
                .fail_critical_msg()
            })
        } else {
            format!("Missing metadata for expected key [{}]", key.name()).fail_critical()
        }
    }

    pub fn expect_field<'a, T: Deserialize<'a>>(&'a self, field: &str) -> Result<T> {
        if let Some(data) = self.0.get(&field.to_string()) {
            serde_json::from_str(data).map_err(|e| {
                format!(
                    "Error occured deserializing expected metadata field [{}], [{}]",
                    field, e
                )
                .fail_critical_msg()
            })
        } else {
            format!("Missing metadata for expected field [{}]", field).fail_critical()
        }
    }

    pub fn put<T: Serialize>(&mut self, key: Key<T>, data: &T) -> Result<Option<String>> {
        match serde_json::to_string(data) {
            Ok(data_str) => Ok(self.0.insert(key.name().to_string(), data_str)),
            Err(e) => format!(
                "Error occurred serializing data for key [{}], Error: [{}]",
                key.name(),
                e
            )
            .fail_critical(),
        }
    }

    pub fn put_field<T: Serialize>(&mut self, field: &str, data: &T) -> Result<Option<String>> {
        match serde_json::to_string(data) {
            Ok(data_str) => Ok(self.0.insert(field.to_string(), data_str)),
            Err(e) => format!(
                "Error occurred serializing data for field [{}], Error: [{}]",
                field, e
            )
            .fail_critical(),
        }
    }

    /// Puts the data if the option is Some, else it does nothing
    pub fn put_optional<T: Serialize, O: Borrow<Option<T>>>(
        &mut self,
        key: Key<T>,
        data: O,
    ) -> Result<Option<String>> {
        if let Some(data_unwrapped) = data.borrow().as_ref() {
            self.put(key, data_unwrapped.borrow())
        } else {
            Ok(None)
        }
    }

    pub fn put_nonempty<T: Serialize, V: Borrow<Vec<T>>>(
        &mut self,
        key: Key<Vec<T>>,
        data: V,
    ) -> Result<Option<String>> {
        if data.borrow().is_empty() {
            Ok(None)
        } else {
            self.put(key, data.borrow())
        }
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

impl Into<HashMap<String, Value>> for Metadata {
    fn into(self) -> HashMap<String, Value> {
        self.0
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

mod test {
    use super::Metadata;
    use std::collections::HashMap;
    use typed_key::{typed_key, Key};

    #[test]
    pub fn hows_it_looking() {
        let mut metadata = Metadata::new();
        let key: Key<usize> = typed_key!("key1");
        let mkey: Key<HashMap<String, String>> = typed_key!("m");
        let metakey: Key<Metadata> = typed_key!("inner_metadata");
        let mut m: HashMap<String, String> = HashMap::new();
        m.insert("what".to_string(), "hey".to_string());
        m.insert("whata".to_string(), "hey".to_string());
        m.insert("foo".to_string(), "bar".to_string());
        metadata.put(key, &343).unwrap();
        metadata.put(mkey, &m).unwrap();
        metadata
            .put(metakey, &{
                let mut metadata = Metadata::new();
                metadata.put(key, &143).unwrap();
                metadata
            })
            .unwrap();
        let result = serde_json::to_string(&metadata).unwrap();
        println!("{}", result);
        // assert_eq!("{\"m\":{\"foo\":\"bar\",\"what\":\"hey\",\"whata\":\"hey\"},\"key1\":343,\"inner_metadata\":{\"key1\":143}}", result);
    }
}
