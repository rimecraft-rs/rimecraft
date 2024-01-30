//! Registry entry types.

use std::{collections::HashSet, ops::Deref};

use parking_lot::RwLock;

use crate::{key::Key, tag::TagKey};

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
mod serde {
    //! Helper module for `serde` support.

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
}

#[cfg(feature = "edcode")]
mod edcode {
    //! Helper module for `edcode` support.

    use std::hash::Hash;

    use rimecraft_edcode::{error::EitherError, Decode, Encode, VarI32};

    use crate::{edcode::Error, ProvideRegistry, Reg};

    use super::{Entry, RefEntry};

    impl<'r, K, T> Encode for RefEntry<K, T>
    where
        K: Hash + Eq + Clone + 'r,
        T: ProvideRegistry<'r, K, T> + 'r,
    {
        type Error = Error<K>;

        fn encode<B>(&self, buf: B) -> Result<(), Self::Error>
        where
            B: rimecraft_edcode::bytes::BufMut,
        {
            let id = Reg::raw_id(
                T::registry()
                    .get(self.key())
                    .ok_or_else(|| Error::InvalidKey(self.key().value().clone()))?,
            );
            VarI32((id + 1) as i32).encode(buf).unwrap();
            Ok(())
        }
    }

    impl<'a, 'r, K, T> Decode for &'a RefEntry<K, T>
    where
        'r: 'a,
        K: 'r,
        T: ProvideRegistry<'r, K, T> + 'r,
    {
        type Output = Self;

        type Error = Error<K>;

        fn decode<B>(buf: B) -> Result<Self::Output, Self::Error>
        where
            B: rimecraft_edcode::bytes::Buf,
        {
            let id = (VarI32::decode(buf)? - 1) as usize;
            T::registry()
                .of_raw(id)
                .map(From::from)
                .ok_or_else(|| Error::InvalidRawId(id))
        }
    }

    impl<'a, 'r, K, T> Encode for Entry<'a, K, T>
    where
        K: Hash + Eq + Clone + 'r,
        T: ProvideRegistry<'r, K, T> + Encode + 'r,
    {
        type Error = EitherError<Error<K>, <T as Encode>::Error>;

        fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
        where
            B: rimecraft_edcode::bytes::BufMut,
        {
            match self {
                Entry::Direct(value) => {
                    VarI32(0).encode(&mut buf).unwrap();
                    value.encode(buf).map_err(EitherError::B)
                }
                Entry::Ref(entry) => entry.encode(buf).map_err(EitherError::A),
            }
        }
    }

    impl<'a, 'r, K, T> Decode for Entry<'a, K, T>
    where
        'r: 'a,
        K: 'r,
        T: ProvideRegistry<'r, K, T> + Decode<Output = T> + 'r,
    {
        type Output = Self;

        type Error = EitherError<Error<K>, <T as Decode>::Error>;

        fn decode<B>(mut buf: B) -> Result<Self::Output, Self::Error>
        where
            B: rimecraft_edcode::bytes::Buf,
        {
            let id = VarI32::decode(&mut buf).map_err(|err| EitherError::A(err.into()))?;
            match id {
                0 => T::decode(buf).map(Entry::Direct).map_err(EitherError::B),
                id => T::registry()
                    .of_raw((id - 1) as usize)
                    .map(|r| Entry::Ref(r.into()))
                    .ok_or_else(|| EitherError::A(Error::InvalidRawId((id - 1) as usize))),
            }
        }
    }
}
