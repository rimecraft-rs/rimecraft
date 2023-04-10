pub mod entry;
pub mod registries;
pub mod registry_keys;
pub mod tag;
pub mod wrapper;

use self::entry::RegistryEntry;
use crate::{
    registry::entry::ReferenceEntry,
    util::{collection::IndexedIterable, Identifier},
};
use datafixerupper::serialization::Lifecycle;
use log::error;
use std::fmt::Display;

pub mod events {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use crate::util::{
        event::{self, Event},
        Identifier,
    };

    pub static INITIALIZE: Lazy<Mutex<Event<(), ()>>> = Lazy::new(|| {
        Mutex::new(Event::new(
            |c, _| {
                for call in c {
                    call(())
                }
            },
            |_| (),
            vec![
                event::default_phase(),
                Identifier::parse("freeze".to_string()).unwrap(),
            ],
        ))
    });
}

pub struct RegistryKey<T> {
    registry: Identifier,
    value: Identifier,
    _none: Option<T>,
}

impl<T> RegistryKey<T> {
    pub fn new(registry: Identifier, value: Identifier) -> Self {
        Self {
            registry,
            value,
            _none: None,
        }
    }

    pub fn of<V>(registry: &RegistryKey<V>, value: Identifier) -> Self
    where
        V: Registry<T>,
    {
        Self::new(registry.value.clone(), value)
    }

    pub fn of_registry<V>(registry: Identifier) -> Self
    where
        T: Registry<V>,
    {
        Self::new(registries::root_key(), registry)
    }

    pub fn is_of<V, A>(&self, registry: &RegistryKey<V>) -> bool
    where
        V: Registry<A>,
    {
        self.registry.eq(registry.get_value())
    }

    pub fn try_cast<V, A>(&self, registry_ref: &RegistryKey<V>) -> Option<RegistryKey<V>>
    where
        V: Registry<A>,
    {
        if self.is_of(registry_ref) {
            Some(RegistryKey {
                registry: self.registry.clone(),
                value: self.value.clone(),
                _none: None,
            })
        } else {
            None
        }
    }

    pub fn get_value(&self) -> &Identifier {
        &self.value
    }

    pub fn get_registry(&self) -> &Identifier {
        &self.registry
    }
}

impl<T> Display for RegistryKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RegistryKey[")?;
        self.registry.fmt(f)?;
        f.write_str(" / ")?;
        self.value.fmt(f)?;
        f.write_str("]")?;
        std::fmt::Result::Ok(())
    }
}

impl<T> Clone for RegistryKey<T> {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            value: self.value.clone(),
            _none: None,
        }
    }
}

impl<T> PartialEq for RegistryKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.registry == other.registry && self.value == other.value
    }
}

pub trait Registry<T>: IndexedIterable<T> {
    fn get_self_key(&self) -> RegistryKey<Self>;

    fn get_raw_id_from_key(&self, key: &RegistryKey<T>) -> Option<usize>;
    fn get_raw_id_from_id(&self, id: &Identifier) -> Option<usize>;

    fn get_from_key<'a>(&'a self, key: &RegistryKey<T>) -> Option<&'a T>;
    fn get_from_id<'a>(&'a self, id: &Identifier) -> Option<&'a T>;

    fn get_entry_lifecycle<'a>(&'a self, entry: usize) -> Option<&Lifecycle>;
    fn get_lifecycle(&self) -> &Lifecycle;

    fn get_ids(&self) -> Vec<&Identifier>;
    fn get_entry_set(&self) -> Vec<(&RegistryKey<T>, &T)>;
    fn get_keys(&self) -> Vec<&RegistryKey<T>>;

    fn contains_id(&self, id: &Identifier) -> bool;
    fn contains(&self, key: &RegistryKey<T>) -> bool;

    fn freeze(&mut self);
}

pub trait DefaultedRegistry<T>: Registry<T> {
    fn get_from_id_default(&self, id: &Identifier) -> &T {
        self.get_from_id(id)
            .unwrap_or(self.get_from_id(self.get_default_id()).unwrap())
    }

    fn get_from_raw_id_default(&self, id: usize) -> &T {
        self.get_from_raw_id(id)
            .unwrap_or(self.get_from_id(self.get_default_id()).unwrap())
    }

    fn get_default_id(&self) -> &Identifier;
}

pub trait MutableRegistry<T>: Registry<T> {
    fn set(
        &mut self,
        id: usize,
        key: RegistryKey<T>,
        object: T,
        lifecycle: Lifecycle,
    ) -> Option<&RegistryEntry<T, Self>>;
    fn add(
        &mut self,
        key: RegistryKey<T>,
        object: T,
        lifecycle: Lifecycle,
    ) -> Option<&RegistryEntry<T, Self>>;
    fn is_empty(&self) -> bool;
}

pub struct SimpleRegistry<T> {
    key: (Identifier, Identifier),
    entries: Vec<(RegistryEntry<T, Self>, Lifecycle)>,
    lifecycle: Lifecycle,
    frozen: bool,
    default_id: Option<Identifier>,
}

impl<T> SimpleRegistry<T> {
    pub fn new(key: RegistryKey<Self>, lifecycle: Lifecycle, default: Option<Identifier>) -> Self {
        Self {
            key: (key.get_registry().clone(), key.get_value().clone()),
            entries: Vec::new(),
            lifecycle,
            frozen: false,
            default_id: default,
        }
    }
}

impl<T> IndexedIterable<T> for SimpleRegistry<T> {
    fn get_from_raw_id(&self, id: usize) -> Option<&T> {
        match self.entries.get(id).map(|t| t.0.value()) {
            Some(Some(a)) => Some(a),
            _ => None,
        }
    }

    fn get_from_raw_id_mut(&mut self, id: usize) -> Option<&mut T> {
        match self.entries.get_mut(id).map(|t| t.0.value_mut()) {
            Some(Some(a)) => Some(a),
            _ => None,
        }
    }

    fn size(&self) -> usize {
        self.entries.len()
    }

    fn vec(&self) -> Vec<&T> {
        self.entries
            .iter()
            .map(|e| e.0.value())
            .filter(|o| o.is_some())
            .map(|o| o.unwrap())
            .collect()
    }

    fn vec_mut(&mut self) -> Vec<&mut T> {
        self.entries
            .iter_mut()
            .map(|e| e.0.value_mut())
            .filter(|o| o.is_some())
            .map(|o| o.unwrap())
            .collect()
    }
}

impl<T> Registry<T> for SimpleRegistry<T> {
    fn get_self_key(&self) -> RegistryKey<Self> {
        RegistryKey::new(self.key.0.clone(), self.key.1.clone())
    }

    fn get_raw_id_from_key(&self, key: &RegistryKey<T>) -> Option<usize> {
        for entry in self.entries.iter().enumerate() {
            if entry.1 .0.get_key().is_some() && entry.1 .0.get_key().unwrap().eq(key) {
                return Some(entry.0);
            }
        }
        None
    }

    fn get_raw_id_from_id(&self, id: &Identifier) -> Option<usize> {
        for entry in self.entries.iter().enumerate() {
            if entry.1 .0.get_key().is_some() && entry.1 .0.get_key().unwrap().get_value().eq(id) {
                return Some(entry.0);
            }
        }
        None
    }

    fn get_from_key<'a>(&'a self, key: &RegistryKey<T>) -> Option<&'a T> {
        for entry in &self.entries {
            if entry.0.get_key().is_some() && entry.0.get_key().unwrap() == key {
                return entry.0.value();
            }
        }
        None
    }

    fn get_from_id<'a>(&'a self, id: &Identifier) -> Option<&'a T> {
        for entry in &self.entries {
            if entry.0.get_key().is_some() && entry.0.get_key().unwrap().get_value() == id {
                return entry.0.value();
            }
        }
        None
    }

    fn get_entry_lifecycle<'a>(&'a self, entry: usize) -> Option<&Lifecycle> {
        self.entries.get(entry).map(|f| &f.1)
    }

    fn get_lifecycle(&self) -> &Lifecycle {
        &self.lifecycle
    }

    fn get_ids(&self) -> Vec<&Identifier> {
        self.entries
            .iter()
            .filter(|t| t.0.get_key().is_some())
            .map(|t| t.0.get_key().unwrap().get_value())
            .collect()
    }

    fn get_entry_set(&self) -> Vec<(&RegistryKey<T>, &T)> {
        self.entries
            .iter()
            .filter(|t| t.0.value().is_some() && t.0.get_key().is_some())
            .map(|t| (t.0.get_key().unwrap(), t.0.value().unwrap()))
            .collect()
    }

    fn get_keys(&self) -> Vec<&RegistryKey<T>> {
        self.entries
            .iter()
            .filter(|t| t.0.get_key().is_some())
            .map(|t| t.0.get_key().unwrap())
            .collect()
    }

    fn contains_id(&self, id: &Identifier) -> bool {
        self.entries
            .iter()
            .any(|p| p.0.get_key().is_some() && p.0.get_key().unwrap().get_value() == id)
    }

    fn contains(&self, key: &RegistryKey<T>) -> bool {
        self.entries
            .iter()
            .any(|p| p.0.get_key().is_some() && p.0.get_key().unwrap() == key)
    }

    fn freeze(&mut self) {
        self.frozen = true;
    }
}

impl<T: PartialEq> MutableRegistry<T> for SimpleRegistry<T> {
    fn set(
        &mut self,
        id: usize,
        key: RegistryKey<T>,
        object: T,
        lifecycle: Lifecycle,
    ) -> Option<&RegistryEntry<T, Self>> {
        if self.frozen || self.entries.len() < id {
            error!("Registry is already frozen (trying to add key {})", key);
            return None;
        }
        if self.entries.len() != id {
            self.entries.remove(id);
        }
        self.entries.insert(
            id,
            (
                {
                    let mut reference = ReferenceEntry::<T, Self>::stand_alone(Some(object));
                    reference.set_registry_key(key);
                    RegistryEntry::Reference(reference)
                },
                lifecycle,
            ),
        );
        self.lifecycle = self.lifecycle + lifecycle;
        self.entries.get(id).map(|t| &t.0)
    }

    fn add(
        &mut self,
        key: RegistryKey<T>,
        object: T,
        lifecycle: Lifecycle,
    ) -> Option<&RegistryEntry<T, Self>> {
        self.set(self.entries.len(), key, object, lifecycle)
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<T: PartialEq> DefaultedRegistry<T> for SimpleRegistry<T> {
    fn get_default_id(&self) -> &Identifier {
        self.default_id.as_ref().unwrap()
    }
}
