use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Metadata(HashMap<String, String>);

impl Metadata {

    fn put_field<T: Serialize>(&mut self, field: &str, data: &T) {
        if let Some(data_str) = serde_json::to_string(data).ok() {
            self.0.insert(field.to_string(), data_str);
        }
    }

    fn get_field<'a, T: Deserialize<'a>>(&'a self, field: &str) -> Option<T> {
        self.0.get(&field.to_string()).and_then(|data|{
            serde_json::from_str(&data).ok()
        })

    }

}
