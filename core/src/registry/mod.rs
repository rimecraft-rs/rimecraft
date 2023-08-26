mod registries;
pub mod tag;

use std::ops::Deref;

use crate::prelude::*;

pub use registries::*;
pub use tag::Key as TagKey;

/// Represents a registration and its id and tags.
pub struct Entry<T> {
    key: Key<T>,
    pub tags: parking_lot::RwLock<Vec<tag::Key<T>>>,
    value: T,
}

impl<T> Entry<T> {
    pub fn key(&self) -> &Key<T> {
        &self.key
    }

    /// If this registration is in target tag.
    pub fn is_in(&self, tag: &tag::Key<T>) -> bool {
        self.tags.read().contains(tag)
    }
}

impl<T> Deref for Entry<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Immutable registry storing entries with mutable tag bindings.
///
/// You're not able to create a registry directly, use a [`Builder`] instead.
pub struct Registry<T> {
    default: Option<usize>,
    entries: Vec<Entry<T>>,
    id_map: std::collections::HashMap<Id, usize>,
    /// Key of this registry.
    pub key: Key<Self>,
    key_map: std::collections::HashMap<Key<T>, usize>,
    /// Tag to entries mapping of this registry.
    pub tags: parking_lot::RwLock<std::collections::HashMap<tag::Key<T>, Vec<usize>>>,
}

impl<T> Registry<T> {
    /// Whether this registry contains an entry with the target registry key.
    pub fn contains_key(&self, key: &Key<T>) -> bool {
        self.key_map.contains_key(key)
    }

    /// Whether this registry contains an entry with the target id.
    pub fn contains_id(&self, id: &Id) -> bool {
        self.id_map.contains_key(id)
    }

    /// Returns the default entry of this reigstry.
    ///
    /// # Panics
    ///
    /// Panic if a default entry don't exist.
    /// See [`Self::is_defaulted`].
    pub fn default_entry(&self) -> (usize, &Entry<T>) {
        let def = self
            .default
            .expect("trying to get a default entry that don't exist");
        (def, self.get_from_raw(def).unwrap())
    }

    /// Get an entry from a [`Key`].
    pub fn get_from_key(&self, key: &Key<T>) -> Option<(usize, &Entry<T>)> {
        self.key_map
            .get(key)
            .map(|e| (*e, self.entries.get(*e).unwrap()))
    }

    /// Get an entry from an [`Id`].
    pub fn get_from_id(&self, id: &Id) -> Option<(usize, &Entry<T>)> {
        self.id_map
            .get(id)
            .map(|e| (*e, self.entries.get(*e).unwrap()))
    }

    /// Get an entry from its raw id.
    pub fn get_from_raw(&self, raw_id: usize) -> Option<&Entry<T>> {
        self.entries.get(raw_id)
    }

    /// Whether a default entry exist in this registry.
    pub fn is_defaulted(&self) -> bool {
        self.default.is_some()
    }

    /// Returns an iterator over the slice of entries.
    pub fn iter(&self) -> std::slice::Iter<'_, Entry<T>> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl<T> std::ops::Index<usize> for Registry<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.get_from_raw(index).unwrap().value
    }
}

impl<T: PartialEq + Eq> crate::util::collections::Indexed<T> for Registry<T> {
    fn raw_id(&self, value: &T) -> Option<usize> {
        self.entries
            .iter()
            .enumerate()
            .find(|e| &e.1.value == value)
            .map(|e| e.0)
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.get_from_raw(index).map(|e| &e.value)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl<T: PartialEq + Eq> crate::util::collections::Indexed<Entry<T>> for Registry<T> {
    fn raw_id(&self, value: &Entry<T>) -> Option<usize> {
        self.entries
            .iter()
            .enumerate()
            .find(|e| e.1 as *const Entry<T> as usize == value as *const Entry<T> as usize)
            .map(|e| e.0)
    }

    fn get(&self, index: usize) -> Option<&Entry<T>> {
        self.get_from_raw(index)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Mutable registry builder for building [`Registry`].
pub struct Builder<T: Registration> {
    entries: Vec<(T, Id)>,
}

impl<T: Registration> Builder<T> {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Register a new value and its id into this builder and return its raw id.
    pub fn register(&mut self, value: T, id: Id) -> Option<usize> {
        if self.entries.iter().any(|e| e.1 == id) {
            None
        } else {
            self.entries.push((value, id));
            Some(self.entries.len() - 1)
        }
    }
}

impl<T: Registration> crate::util::Freeze<Registry<T>> for Builder<T> {
    type Opts = (Key<Registry<T>>, Option<Id>);

    fn build(self, opts: Self::Opts) -> Registry<T> {
        let entries = self
            .entries
            .into_iter()
            .enumerate()
            .map(|mut e| {
                e.1 .0.accept(e.0);
                Entry {
                    value: e.1 .0,
                    key: Key::new(opts.0, e.1 .1.clone()),
                    tags: parking_lot::RwLock::new(Vec::new()),
                }
            })
            .collect::<Vec<_>>();

        let id_map = {
            let mut map = std::collections::HashMap::new();
            for e in entries.iter().enumerate() {
                map.insert(e.1.key.value().clone(), e.0);
            }
            map
        };

        Registry {
            default: opts.1.map(|e| id_map.get(&e).copied()).flatten(),
            key_map: {
                let mut map = std::collections::HashMap::new();
                for e in entries.iter().enumerate() {
                    map.insert(e.1.key.clone(), e.0);
                }
                map
            },
            entries,
            id_map,
            key: opts.0,
            tags: parking_lot::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

/// Registration for storing raw_id.
pub trait Registration {
    /// Accept a raw id.
    fn accept(&mut self, id: usize);
    /// Return the raw id.
    fn raw_id(&self) -> usize;
}

pub trait RegistryAccess: Sized {
    fn registry() -> &'static Registry<Self>;
}

static KEYS_CACHE: crate::collections::Caches<(Id, Id)> = crate::collections::Caches::new();

/// Represents a key for a value in a registry in a context where
/// a root registry is available.
///
/// This type is cheap to clone.
pub struct Key<T> {
    _type: std::marker::PhantomData<T>,
    // (reg, value)
    inner: crate::Ref<'static, (Id, Id)>,
}

impl<T> Key<T> {
    pub fn new(registry: Key<Registry<T>>, value: Id) -> Self {
        Self {
            inner: crate::Ref(KEYS_CACHE.get((registry.inner.0 .1.clone(), value))),
            _type: std::marker::PhantomData,
        }
    }

    /// Whether this registry key belongs to the given registry.
    pub fn is_of<E>(&self, reg: &Key<Registry<E>>) -> bool {
        self.inner.0 .0 == reg.inner.1
    }

    /// Return `Some(_)` if the key is of reg, otherwise `None`.
    pub fn cast<E>(&self, reg: &Key<Registry<E>>) -> Option<Key<E>> {
        if self.is_of(&reg) {
            Some(Key {
                inner: self.inner,
                _type: std::marker::PhantomData,
            })
        } else {
            None
        }
    }

    /// Value of this key.
    pub fn value(&self) -> &'static Id {
        &self.inner.0 .0
    }

    /// Registry of this key.
    pub fn reg(&self) -> &'static Id {
        &self.inner.0 .1
    }
}

impl<T> Key<Registry<T>> {
    /// Creates a registry key for a registry in the root registry
    /// with an identifier for the registry.
    pub fn of_reg(reg: Id) -> Self {
        Self {
            inner: crate::Ref(KEYS_CACHE.get((registries::root_key(), reg))),
            _type: std::marker::PhantomData,
        }
    }
}

impl<T> std::fmt::Debug for Key<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RegistryKey[")?;
        std::fmt::Display::fmt(&self.inner.0 .0, f)?;
        f.write_str(" / ")?;
        self.inner.0 .1.fmt(f)?;
        f.write_str("]")
    }
}

impl<T> Copy for Key<T> {}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for Key<T> {}

impl<T> std::hash::Hash for Key<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

/// Freezeable registry for building and freezing registries,
/// just like what MCJE's `Registry` do.
pub type Freezer<T> = crate::util::Freezer<Registry<T>, Builder<T>>;
