use std::{fmt::Display, marker::PhantomData};

use crate::util::Identifier;

use super::{Registry, RegistryKey};

pub struct TagKey<T> {
    registry: RegistryKey<Registry<T>>,
    id: Identifier,
    _phantom: PhantomData<T>,
}

impl<T> TagKey<T> {
    pub fn new(registry: RegistryKey<Registry<T>>, id: Identifier) -> Self {
        Self {
            registry,
            id,
            _phantom: PhantomData,
        }
    }

    pub fn get_id(&self) -> &Identifier {
        &self.id
    }

    pub fn get_registry(&self) -> &RegistryKey<Registry<T>> {
        &self.registry
    }

    pub fn is_of(&self, other_registry: &RegistryKey<Registry<T>>) -> bool {
        self.registry.is_of(other_registry)
    }

    pub fn try_cast<E>(&self, registry_key: RegistryKey<Registry<E>>) -> TagKey<E> {
        TagKey::<E>::new(registry_key, self.id.clone())
    }
}

impl<T> PartialEq for TagKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.registry == other.registry
    }
}

impl<T> Clone for TagKey<T> {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            id: self.id.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> Display for TagKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TagKey[")?;
        self.registry.get_value().fmt(f)?;
        f.write_str(" / ")?;
        self.id.fmt(f)?;
        f.write_str("]")?;
        std::fmt::Result::Ok(())
    }
}
