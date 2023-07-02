mod registries;
pub mod tag;

use std::ops::Deref;

use crate::prelude::*;

pub use registries::*;

/// Represents a registration and its id and tags.
pub struct Holder<T> {
    key: RegistryKey<T>,
    pub tags: tokio::sync::RwLock<Vec<tag::TagKey<T>>>,
    value: T,
}

impl<T> Holder<T> {
    pub fn key(&self) -> &RegistryKey<T> {
        &self.key
    }

    /// If this registration is in target tag.
    pub fn is_in(&self, tag: &tag::TagKey<T>) -> bool {
        self.tags.blocking_read().contains(tag)
    }

    /// If this registration is in target tag, executed in async.
    pub async fn async_is_in(&self, tag: &tag::TagKey<T>) -> bool {
        self.tags.read().await.contains(tag)
    }
}

impl<T> Deref for Holder<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Immutable registry with mutable tag bindings.
pub struct Registry<T> {
    default: Option<usize>,
    entries: Vec<Holder<T>>,
    id_map: hashbrown::HashMap<Identifier, usize>,
    pub key: RegistryKey<Self>,
    key_map: hashbrown::HashMap<RegistryKey<T>, usize>,
    pub tags: tokio::sync::RwLock<hashbrown::HashMap<tag::TagKey<T>, Vec<usize>>>,
}

impl<T> Registry<T> {
    pub fn iter(&self) -> std::slice::Iter<'_, Holder<T>> {
        self.entries.iter()
    }

    pub fn get_from_raw(&self, raw_id: usize) -> Option<&Holder<T>> {
        self.entries.get(raw_id)
    }

    pub fn get(&self, key: &RegistryKey<T>) -> Option<(usize, &Holder<T>)> {
        self.key_map
            .get(key)
            .map(|e| (*e, self.entries.get(*e).unwrap()))
    }

    pub fn get_from_id(&self, id: &Identifier) -> Option<(usize, &Holder<T>)> {
        self.id_map
            .get(id)
            .map(|e| (*e, self.entries.get(*e).unwrap()))
    }

    pub fn contains(&self, key: &RegistryKey<T>) -> bool {
        self.key_map.contains_key(key)
    }

    pub fn contains_id(&self, id: &Identifier) -> bool {
        self.id_map.contains_key(id)
    }

    pub fn default(&self) -> (usize, &Holder<T>) {
        let def = self.default.unwrap();
        (def, self.get_from_raw(def).unwrap())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_defaulted(&self) -> bool {
        self.default.is_some()
    }
}

impl<T> std::ops::Index<usize> for Registry<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries.get(index).unwrap().value
    }
}

/// Mutable registry builder for building [`Registry`].
pub struct Builder<T: Registration> {
    entries: Vec<(T, Identifier)>,
}

impl<T: Registration> Builder<T> {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Put a new value and its id into this registry builder and return its raw id.
    pub fn register(&mut self, value: T, id: Identifier) -> anyhow::Result<usize> {
        if self.entries.iter().any(|e| e.1 == id) {
            Err(anyhow::anyhow!("Registration with id {id} already exist!"))
        } else {
            self.entries.push((value, id));
            Ok(self.entries.len() - 1)
        }
    }

    /// Build this builder into a [`Registry`] with target registry key.
    pub fn build(self, reg: RegistryKey<Registry<T>>, def: Option<Identifier>) -> Registry<T> {
        let entries = self
            .entries
            .into_iter()
            .enumerate()
            .map(|mut e| {
                e.1 .0.accept(e.0);
                Holder {
                    value: e.1 .0,
                    key: RegistryKey::new(&reg, e.1 .1.clone()),
                    tags: tokio::sync::RwLock::new(Vec::new()),
                }
            })
            .collect::<Vec<_>>();

        let id_map = {
            let mut map = hashbrown::HashMap::new();
            for e in entries.iter().enumerate() {
                map.insert(e.1.key.value().clone(), e.0);
            }
            map
        };

        Registry {
            default: def.map(|e| id_map.get(&e).copied()).flatten(),
            key_map: {
                let mut map = hashbrown::HashMap::new();
                for e in entries.iter().enumerate() {
                    map.insert(e.1.key.clone(), e.0);
                }
                map
            },
            entries,
            id_map,
            key: reg,
            tags: tokio::sync::RwLock::new(hashbrown::HashMap::new()),
        }
    }
}

/// Registratio for storing raw_id.
pub trait Registration {
    /// Accept a raw id.
    fn accept(&mut self, id: usize);
    /// Return the raw id.
    fn raw_id(&self) -> usize;
}

pub trait RegistryAccess: Sized {
    fn registry() -> &'static Registry<Self>;
}

/// Represents a key for a value in a registry in a context where
/// a root registry is available.
pub struct RegistryKey<T> {
    _type: std::marker::PhantomData<T>,
    /// (reg, value)
    inner: std::sync::Arc<(Identifier, Identifier)>,
}

impl<T> RegistryKey<T> {
    pub fn new(registry: &RegistryKey<Registry<T>>, value: Identifier) -> Self {
        Self {
            inner: std::sync::Arc::new((registry.inner.1.clone(), value)),
            _type: std::marker::PhantomData,
        }
    }

    /// Whether this registry key belongs to the given registry.
    pub fn is_of<E>(&self, reg: &RegistryKey<Registry<E>>) -> bool {
        self.inner.0 == reg.inner.1
    }

    /// Return `Some(_)` if the key is of reg, otherwise `None`.
    pub fn cast<E>(&self, reg: &RegistryKey<Registry<E>>) -> Option<RegistryKey<E>> {
        if self.is_of(&reg) {
            Some(RegistryKey {
                inner: std::sync::Arc::clone(&self.inner),
                _type: std::marker::PhantomData,
            })
        } else {
            None
        }
    }

    /// Value of this key.
    pub fn value(&self) -> &Identifier {
        &self.inner.0
    }

    /// Registry of this key.
    pub fn reg(&self) -> &Identifier {
        &self.inner.1
    }
}

impl<T> RegistryKey<Registry<T>> {
    /// Creates a registry key for a registry in the root registry
    /// with an identifier for the registry.
    pub fn of_reg(reg: Identifier) -> Self {
        Self {
            inner: std::sync::Arc::new((registries::root_key(), reg)),
            _type: std::marker::PhantomData,
        }
    }
}

impl<T> std::fmt::Display for RegistryKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RegistryKey[")?;
        self.inner.0.fmt(f)?;
        f.write_str(" / ")?;
        self.inner.1.fmt(f)?;
        f.write_str("]")
    }
}

impl<T> Clone for RegistryKey<T> {
    fn clone(&self) -> Self {
        Self {
            inner: std::sync::Arc::clone(&self.inner),
            _type: std::marker::PhantomData,
        }
    }
}

impl<T> PartialEq for RegistryKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for RegistryKey<T> {}

impl<T> std::hash::Hash for RegistryKey<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

/// Lazy registry for building and freezing registries,
/// just like what vanilla Minecraft's `Registry` do.
///
/// Can be used in static instances.
pub struct Lazy<T: Registration> {
    builder: parking_lot::Mutex<Option<Builder<T>>>,
    registry: once_cell::sync::OnceCell<Registry<T>>,
}

impl<T: Registration> Lazy<T> {
    pub const fn new() -> Self {
        Self {
            builder: parking_lot::Mutex::new(None),
            registry: once_cell::sync::OnceCell::new(),
        }
    }

    pub fn register(&self, value: T, id: Identifier) -> anyhow::Result<usize> {
        self.builder
            .lock()
            .as_mut()
            .expect("Registry has already been freezed")
            .register(value, id)
    }

    /// Freeze this registry into an immutable [`Registry`] instance
    /// with a registry key.
    pub fn freeze(
        &self,
        registry: RegistryKey<Registry<T>>,
        default_registration: Option<Identifier>,
    ) {
        if self.registry.get().is_some() {
            return;
        }

        let registry = self
            .builder
            .lock()
            .take()
            .unwrap()
            .build(registry, default_registration);
        self.registry.get_or_init(|| registry);
    }
}

impl<T: Registration> Deref for Lazy<T> {
    type Target = Registry<T>;

    fn deref(&self) -> &Self::Target {
        self.registry.get().expect("Registry has not been built")
    }
}
