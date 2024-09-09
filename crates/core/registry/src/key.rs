//! Registry key related types.

use std::{hash::Hash, marker::PhantomData};

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

    #[doc(hidden)]
    #[inline]
    pub fn cast<V>(self) -> Key<K, V> {
        Key {
            registry: self.registry,
            value: self.value,
            _marker: PhantomData,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn cast_ref<V>(&self) -> &Key<K, V> {
        unsafe { &*std::ptr::from_ref(self).cast::<Key<K, V>>() }
    }
}

impl<K, T> Key<K, T>
where
    K: Root,
{
    /// Creates a new key with the root registry.
    #[inline]
    pub fn with_root(value: K) -> Self {
        Self::new(K::root(), value)
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

/// Trait for presenting root registry key ID.
pub trait Root: Sized {
    /// Gets the root registry key ID.
    fn root() -> Self;
}

#[cfg(feature = "serde")]
mod serde {
    use crate::ProvideRegistry;

    use super::Key;

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
}

/// Helper module for `edcode` support.
#[cfg(feature = "edcode")]
pub mod edcode {

    use edcode2::{Decode, Encode};

    use crate::ProvideRegistry;

    use super::{Key, Root};

    impl<K, T, B> Encode<B> for Key<K, T>
    where
        K: Encode<B>,
    {
        #[inline]
        fn encode(&self, buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            self.value.encode(buf)
        }
    }

    impl<'r, 'de, K, T, B> Decode<'de, B> for Key<K, T>
    where
        K: Decode<'de, B> + Clone + 'r,
        T: ProvideRegistry<'r, K, T> + 'r,
    {
        #[inline]
        fn decode(buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
            let value = K::decode(buf)?;
            Ok(Key::new(T::registry().key.value.to_owned(), value))
        }
    }

    /// Wrapper for registry reference keys.
    #[derive(Debug, Clone, Copy)]
    pub struct RegRef<T>(pub T);

    impl<K, T, B> Encode<B> for RegRef<&Key<K, T>>
    where
        K: Encode<B>,
    {
        #[inline]
        fn encode(&self, buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            self.0.value.encode(buf)
        }
    }

    impl<K, T, B> Encode<B> for RegRef<Key<K, T>>
    where
        K: Encode<B>,
    {
        #[inline]
        fn encode(&self, buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            RegRef(&self.0).encode(buf)
        }
    }

    impl<'r, 'de, K, T, B> Decode<'de, B> for RegRef<Key<K, T>>
    where
        K: Decode<'de, B> + Clone + Root + 'r,
    {
        #[inline]
        fn decode(buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
            Ok(Self(Key::new(K::root(), K::decode(buf)?)))
        }
    }
}
