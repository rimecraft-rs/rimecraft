use std::hash::Hash;

use properties::PropertyMap;
use uuid::Uuid;

pub mod properties;

#[derive(Debug)]
pub struct GameProfile {
    id: Option<Uuid>,
    name: String,
    properties: PropertyMap,
    legacy: bool,
}

impl GameProfile {
    pub fn new(id: Option<Uuid>, name: String) -> Option<Self> {
        if id.is_none() && name.is_empty() {
            None
        } else {
            Some(Self {
                id,
                name,
                properties: PropertyMap::new(),
                legacy: bool::default(),
            })
        }
    }

    pub fn get_id(&self) -> Option<Uuid> {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_properties(&self) -> &PropertyMap {
        &self.properties
    }

    pub fn get_properties_mut(&mut self) -> &mut PropertyMap {
        &mut self.properties
    }

    pub fn is_complete(&self) -> bool {
        self.id.is_some() && !self.name.is_empty()
    }

    pub fn is_legacy(&self) -> bool {
        self.legacy
    }
}

impl PartialEq for GameProfile {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

impl Hash for GameProfile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
    }
}
