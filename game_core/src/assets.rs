use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::model::curio_action::CurioAction;

pub trait Asset: Serialize + for<'de> Deserialize<'de> {
    const SUB_EXTENSION: &'static str;
}

pub struct AssetDictionary<T: Asset>(HashMap<String, Arc<T>>);

impl Asset for CurioAction {
    const SUB_EXTENSION: &'static str = "actions";
}
