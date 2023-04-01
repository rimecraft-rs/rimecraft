use std::collections::hash_map::RandomState;

use multimap::MultiMap;

#[derive(Debug)]
pub struct PropertyMap {
    pub map: MultiMap<String, Property, RandomState>,
}

impl PropertyMap {
    pub fn new() -> Self {
        Self {
            map: MultiMap::new(),
        }
    }
}

impl Default for PropertyMap {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Property {
    name: String,
    value: String,
    signature: Option<String>,
}

impl Property {
    pub fn new(name: String, value: String, signature: String) -> Self {
        Self {
            name,
            value,
            signature: Some(signature),
        }
    }

    pub fn new_without_signature(name: String, value: String) -> Self {
        Self {
            name,
            value,
            signature: None,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn get_signature(&self) -> Option<&str> {
        self.signature.as_deref()
    }

    pub fn has_signature(&self) -> bool {
        self.signature.is_some()
    }
}
