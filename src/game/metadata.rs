use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use typed_key::{Key};
use super::error::{ErrorMsg as _, Result};

#[derive(Serialize, Deserialize)]
pub struct Metadata(HashMap<String, String>);

impl Default for Metadata {
    fn default() -> Self {
        Metadata(HashMap::new())
    }
}

impl Metadata {

    pub fn new() -> Self {
        Metadata::default()
    }

    pub fn get<'a, T: Deserialize<'a>>(&'a self, key: &Key<T>) -> Result<Option<T>> {
        self.0.get(&key.name().to_string()).map(|data|{
            serde_json::from_str(&data).map_err(|e| {
                format!("Error occured deserializing metadata key [{}], [{}]", key.name(), e).fail_critical_msg()
            })
        }).transpose()
    }

    pub fn get_field<'a, T: Deserialize<'a>>(&'a self, field: &str) -> Result<Option<T>> {
        self.0.get(&field.to_string()).map(|data|{
            serde_json::from_str(&data).map_err(|e| {
                format!("Error occured deserializing metadata field [{}], [{}]", field, e).fail_critical_msg()
            })
        }).transpose()
    }

    pub fn expect<'a, T: Deserialize<'a>>(&'a self, key: &Key<T>) -> Result<T> {
        if let Some(data) = self.0.get(&key.name().to_string()) {
            serde_json::from_str(&data).map_err(|e| {
                format!("Error occured deserializing expected metadata key [{}], [{}]", key.name(), e).fail_critical_msg()
            })
        } else {
            format!("Missing metadata for expected key [{}]", key.name()).fail_critical()
        }
    }

    pub fn expect_field<'a, T: Deserialize<'a>>(&'a self, field: &str) -> Result<T> {
        if let Some(data) = self.0.get(&field.to_string()) {
            serde_json::from_str(&data).map_err(|e| {
                format!("Error occured deserializing expected metadata field [{}], [{}]", field, e).fail_critical_msg()
            })
        } else {
            format!("Missing metadata for expected field [{}]", field).fail_critical()
        }
    }

    pub fn put<T: Serialize>(&mut self, key: &Key<T>, data: &T) -> Result<Option<String>> {
        match serde_json::to_string(data) {
            Ok(data_str) => Ok(self.0.insert(key.name().to_string(), data_str)),
            Err(e) => format!("Error occurred serializing data for key [{}], Error: [{}]", key.name(), e).fail_critical()
        }
    }

    pub fn put_field<T: Serialize>(&mut self, field: &str, data: &T) -> Result<Option<String>> {
        match serde_json::to_string(data) {
            Ok(data_str) => Ok(self.0.insert(field.to_string(), data_str)),
            Err(e) => format!("Error occurred serializing data for field [{}], Error: [{}]", field, e).fail_critical()
        }
    }

    /// Puts the data if the option is Some, else it does nothing
    pub fn put_optional<T: Serialize>(&mut self, key: &Key<T>, data: Option<&T>) -> Result<Option<String>> {
        if let Some(data_unwrapped) = data {
            self.put(key, data_unwrapped)
        } else {
            Ok(None)
        }
    }

}
