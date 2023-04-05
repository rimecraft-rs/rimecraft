use std::fmt::Display;

use crate::util::Identifier;

use super::{RegistryKey, Registry};

pub struct TagKey<T> {
    // TODO: here
    pub registry: Option<T>,
    pub id: Identifier,
}

impl<T> PartialEq for TagKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
