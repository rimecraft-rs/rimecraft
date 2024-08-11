//! Registry entry types.

use std::{collections::HashSet, ops::Deref};

use parking_lot::RwLock;

use crate::{key::Key, tag::TagKey};

/// Type holds a value that can be registered
/// in a registry.
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum Entry<'a, K, T> {
    /// Holds the value directly.
    Direct(T),
    /// Holds the value by reference.
    Ref(&'a RefEntry<K, T>),
}

impl<K, T> Entry<'_, K, T> {
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

impl<K, T> From<T> for Entry<'_, K, T> {
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

impl<K, T> Deref for TagsGuard<'_, K, T> {
    type Target = HashSet<TagKey<K, T>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K: std::fmt::Debug, T> std::fmt::Debug for TagsGuard<'_, K, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TagsGuard").field(&self.inner).finish()
    }
}

#[cfg(feature = "serde")]
mod serde {
    use std::hash::Hash;

    use crate::ProvideRegistry;

    use super::RefEntry;

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

    impl<'a, 'de, K, T> serde::Deserialize<'de> for &'a RefEntry<K, T>
    where
        K: serde::Deserialize<'de> + Hash + Eq + 'a,
        T: ProvideRegistry<'a, K, T> + 'a,
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
                .ok_or_else(|| serde::de::Error::custom("unknown registry key"))
        }
    }
}

#[cfg(feature = "edcode")]
mod edcode {
    use std::{fmt::Debug, hash::Hash};

    use edcode2::{Buf, BufExt, BufMut, BufMutExt, Decode, Encode};

    use crate::{ProvideRegistry, Reg};

    use super::{Entry, RefEntry};

    impl<'r, K, T, B> Encode<B> for RefEntry<K, T>
    where
        K: Hash + Eq + Clone + Debug + 'r,
        T: ProvideRegistry<'r, K, T> + 'r,
        B: BufMut,
    {
        fn encode(&self, mut buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            let id = Reg::raw_id(T::registry().get(self.key()).ok_or_else(|| {
                edcode2::BoxedError::<'static>::from(format!(
                    "unknown registry key {:?}",
                    self.key()
                ))
            })?);
            buf.put_variable((id + 1) as u32);
            Ok(())
        }
    }

    impl<'a, 'r, 'de, K, T, B> Decode<'de, B> for &'a RefEntry<K, T>
    where
        'r: 'a,
        K: 'r,
        T: ProvideRegistry<'r, K, T> + 'r,
        B: Buf,
    {
        fn decode(mut buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
            let id = buf.get_variable::<u32>() as usize - 1;
            T::registry()
                .of_raw(id)
                .map(From::from)
                .ok_or_else(|| format!("unknown registry id: {}", id).into())
        }
    }

    impl<'r, K, T, B> Encode<B> for Entry<'_, K, T>
    where
        K: Hash + Eq + Clone + Debug + 'r,
        T: ProvideRegistry<'r, K, T> + Encode<B> + 'r,
        B: BufMut,
    {
        fn encode(&self, mut buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            match self {
                Entry::Direct(value) => {
                    buf.put_variable(0i32);
                    value.encode(buf)
                }
                Entry::Ref(entry) => entry.encode(buf),
            }
        }
    }

    impl<'a, 'r, 'de, K, T, B> Decode<'de, B> for Entry<'a, K, T>
    where
        'r: 'a,
        K: 'r,
        T: ProvideRegistry<'r, K, T> + Decode<'de, B> + 'r,
        B: Buf,
    {
        fn decode(mut buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
            let id = buf.get_variable::<u32>() as usize;
            match id {
                0 => T::decode(buf).map(Entry::Direct),
                id => T::registry()
                    .of_raw(id - 1)
                    .map(|r| Entry::Ref(r.into()))
                    .ok_or_else(|| format!("unknown registry id: {}", id - 1).into()),
            }
        }
    }
}
