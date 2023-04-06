use std::fmt::Display;

use crate::util::Identifier;

use super::{Registry, RegistryKey};

pub struct TagKey<T, R: Registry<T>> {
    registry: RegistryKey<R>,
    id: Identifier,
    _none: Option<T>,
}

impl<T, R: Registry<T>> TagKey<T, R> {
    pub fn new(registry: RegistryKey<R>, id: Identifier) -> Self {
        Self {
            registry,
            id,
            _none: None,
        }
    }

    pub fn get_id(&self) -> &Identifier {
        &self.id
    }

    pub fn get_registry(&self) -> &RegistryKey<R> {
        &self.registry
    }

    pub fn is_of<R2>(&self, other_registry: &RegistryKey<R2>) -> bool
    where
        R2: Registry<T>,
    {
        self.registry.is_of(other_registry)
    }

    pub fn try_cast<E, R2>(&self, registry_key: RegistryKey<R2>) -> TagKey<E, R2>
    where
        R2: Registry<E>,
    {
        TagKey::<E, R2>::new(registry_key, self.id.clone())
    }
}

impl<T, R: Registry<T>> PartialEq for TagKey<T, R> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.registry == other.registry
    }
}

impl<T, R: Registry<T>> Clone for TagKey<T, R> {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            id: self.id.clone(),
            _none: None,
        }
    }
}

impl<T, R: Registry<T>> Display for TagKey<T, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TagKey[")?;
        self.registry.get_value().fmt(f)?;
        f.write_str(" / ")?;
        self.id.fmt(f)?;
        f.write_str("]")?;
        std::fmt::Result::Ok(())
    }
}
