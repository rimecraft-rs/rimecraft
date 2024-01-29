//! Registry key related types.

use std::{hash::Hash, marker::PhantomData};

use crate::ProvideRegistry;

/// A key for a value in a registry in a context
/// where a root registry is available.
pub struct Key<K, T> {
    /// The id of the registry in the root registry.
    registry: K,
    /// The id of the value in the registry specified
    /// by [`Self::registry`].
    value: K,

    _marker: PhantomData<T>,
}

impl<K, T> Key<K, T> {
    /// Creates a new key.
    #[inline]
    pub const fn new(registry: K, value: K) -> Self {
        Self {
            registry,
            value,
            _marker: PhantomData,
        }
    }

    /// Gets the id of the value in the registry.
    #[inline]
    pub fn value(&self) -> &K {
        &self.value
    }

    /// Gets the id of the registry in the root registry.
    #[inline]
    pub fn registry(&self) -> &K {
        &self.registry
    }
}

impl<K: Hash, T> Hash for Key<K, T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.registry.hash(state);
        self.value.hash(state);
    }
}

impl<K: Clone, T> Clone for Key<K, T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            value: self.value.clone(),
            _marker: PhantomData,
        }
    }
}

impl<K: Copy, T> Copy for Key<K, T> {}

impl<K: std::fmt::Debug, T> std::fmt::Debug for Key<K, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RegistryKey")
            .field(&self.registry)
            .field(&self.value)
            .finish()
    }
}

impl<K: PartialEq, T> PartialEq for Key<K, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.registry == other.registry && self.value == other.value
    }
}

impl<K: Eq, T> Eq for Key<K, T> {}

impl<K, T> AsRef<K> for Key<K, T> {
    #[inline]
    fn as_ref(&self) -> &K {
        &self.value
    }
}

#[cfg(feature = "serde")]
impl<K, T> serde::Serialize for Key<K, T>
where
    K: serde::Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'r, 'de, K, T> serde::Deserialize<'de> for Key<K, T>
where
    K: serde::Deserialize<'de> + Clone + 'r,
    T: ProvideRegistry<'r, K, T> + 'r,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let registry = T::registry();
        let value = K::deserialize(deserializer)?;
        Ok(Self::new(registry.key.value.clone(), value))
    }
}
