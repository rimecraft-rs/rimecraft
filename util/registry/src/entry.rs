//! Registry entry types.

use std::{collections::HashSet, hash::Hash, ops::Deref};

use parking_lot::RwLock;

use crate::{key::Key, tag::TagKey, ProvideRegistry};

/// Type holds a value that can be registered
/// in a registry.
#[derive(Debug)]
pub enum Entry<'a, K, T> {
    /// Holds the value directly.
    Direct(T),
    /// Holds the value by reference.
    Ref(&'a RefEntry<K, T>),
}

impl<'a, K, T> Entry<'a, K, T> {
    /// Gets the containing value of this entry.
    #[inline]
    pub fn value(&self) -> Option<&T> {
        match self {
            Entry::Direct(value) => Some(value),
            Entry::Ref(entry) => entry.value(),
        }
    }

    /// Gets the key of this entry.
    ///
    /// Returns `None` if the entry is a direct value.
    #[inline]
    pub fn key(&self) -> Option<&Key<K, T>> {
        match self {
            Entry::Direct(_) => None,
            Entry::Ref(entry) => Some(entry.key()),
        }
    }

    /// Gets the id of this entry.
    ///
    /// Returns `None` if the entry is a direct value.
    #[inline]
    pub fn id(&self) -> Option<&K> {
        self.key().map(Key::value)
    }
}

impl<'a, K, T> From<T> for Entry<'a, K, T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::Direct(value)
    }
}

/// Registry entry holds the value by reference.
///
/// The value is previously registered in a `Registry`, so
/// they can be referred to their registry keys.
///
/// This type also holds the entry's tags.
#[derive(Debug)]
pub struct RefEntry<K, T> {
    pub(crate) raw: usize,
    pub(crate) key: Key<K, T>,
    pub(crate) value: Option<T>,
    pub(crate) tags: RwLock<HashSet<TagKey<K, T>>>,
}

impl<K, T> RefEntry<K, T> {
    /// Gets the raw id of this entry.
    #[inline]
    pub fn raw_id(&self) -> usize {
        self.raw
    }

    /// Gets the containing value of this entry.
    #[inline]
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Gets the key of this entry.
    #[inline]
    pub fn key(&self) -> &Key<K, T> {
        &self.key
    }

    /// Gets the tags of this entry.
    #[inline]
    pub fn tags(&self) -> TagsGuard<'_, K, T> {
        TagsGuard {
            inner: self.tags.read(),
        }
    }
}

/// Guard of tags.
pub struct TagsGuard<'a, K, T> {
    inner: parking_lot::RwLockReadGuard<'a, HashSet<TagKey<K, T>>>,
}

impl<'a, K, T> Deref for TagsGuard<'a, K, T> {
    type Target = HashSet<TagKey<K, T>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, K: std::fmt::Debug, T> std::fmt::Debug for TagsGuard<'a, K, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TagsGuard").field(&self.inner).finish()
    }
}

#[cfg(feature = "serde")]
impl<K, T> serde::Serialize for RefEntry<K, T>
where
    K: serde::Serialize,
{
    /// Serializes the registry entry using the ID.
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.key.value().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'a, 'r, 'de, K, T> serde::Deserialize<'de> for &'a RefEntry<K, T>
where
    'r: 'a,
    K: serde::Deserialize<'de> + Hash + Eq + std::fmt::Debug + 'r,
    T: ProvideRegistry<'r, K, T> + 'r,
{
    /// Deserializes the registry entry using the ID.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = K::deserialize(deserializer)?;
        T::registry()
            .get(&id)
            .map(From::from)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown registry key {id:?}")))
    }
}
