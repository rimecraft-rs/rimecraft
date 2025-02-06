//! Tag related types.

use std::{collections::HashMap, hash::Hash};

use crate::{key::Key, Registry};

/// Key of a tag.
pub struct TagKey<K, T> {
    /// The registry reference.
    pub registry: Key<K, Registry<K, T>>,
    /// The tag id.
    pub id: K,
}

impl<K: Hash, T> Hash for TagKey<K, T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.registry.hash(state);
        self.id.hash(state);
    }
}

impl<K: PartialEq, T> PartialEq for TagKey<K, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.registry == other.registry && self.id == other.id
    }
}

impl<K: Eq, T> Eq for TagKey<K, T> {}

impl<K: Clone, T> Clone for TagKey<K, T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            id: self.id.clone(),
        }
    }
}

impl<K: Copy, T> Copy for TagKey<K, T> {}

impl<K: std::fmt::Debug, T> std::fmt::Debug for TagKey<K, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TagKey")
            .field(&self.registry.value())
            .field(&self.id)
            .finish()
    }
}

/// Tags of a registry.
#[derive(Debug)]
pub struct Tags<'r, K, T> {
    pub(crate) inner: parking_lot::RwLockReadGuard<'r, HashMap<TagKey<K, T>, Vec<usize>>>,
    pub(crate) registry: &'r Registry<K, T>,
}

impl<K, T> Tags<'_, K, T> {
    /// Gets an iterator over the tags.
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, T> {
        Iter {
            inner: self.inner.iter(),
            registry: self.registry,
        }
    }
}

impl<'a: 'a, K, T> IntoIterator for &'a Tags<'_, K, T> {
    type Item = (&'a TagKey<K, T>, crate::Entries<'a, K, T>);

    type IntoIter = Iter<'a, K, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator of tags.
#[derive(Debug)]
pub struct Iter<'a, K, T> {
    inner: std::collections::hash_map::Iter<'a, TagKey<K, T>, Vec<usize>>,
    registry: &'a Registry<K, T>,
}

impl<'a, K, T> Iterator for Iter<'a, K, T> {
    type Item = (&'a TagKey<K, T>, crate::Entries<'a, K, T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(t, v)| {
            (
                t,
                crate::Entries {
                    inner: crate::EntriesInner::Raw {
                        registry: self.registry,
                        iter: v.iter(),
                    },
                },
            )
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Helper module for `serde` support.
#[cfg(feature = "serde")]
pub mod serde {
    use std::{marker::PhantomData, str::FromStr};

    use local_cx::{serde::DeserializeWithCx, LocalContext};

    use crate::Registry;

    use super::TagKey;

    /// `TagKey` serialize and deserailize wrapper
    /// without `#` prefix.
    #[derive(Debug, Clone, Copy)]
    pub struct Unprefixed<T>(pub T);

    impl<K, T> serde::Serialize for Unprefixed<&TagKey<K, T>>
    where
        K: serde::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.id.serialize(serializer)
        }
    }

    impl<K, T> serde::Serialize for Unprefixed<TagKey<K, T>>
    where
        K: serde::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Unprefixed(&self.0).serialize(serializer)
        }
    }

    impl<'de, 'r, K, T: 'r, Cx> DeserializeWithCx<'de, Cx> for Unprefixed<TagKey<K, T>>
    where
        K: DeserializeWithCx<'de, Cx> + Clone + 'r,
        Cx: LocalContext<&'r Registry<K, T>>,
    {
        fn deserialize_with_cx<D>(
            deserializer: local_cx::WithLocalCx<D, Cx>,
        ) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let registry = deserializer.local_cx.acquire();
            Ok(Unprefixed(TagKey {
                registry: registry.key.clone(),
                id: K::deserialize_with_cx(deserializer)?,
            }))
        }
    }

    impl<K, T> serde::Serialize for TagKey<K, T>
    where
        K: ToString,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            format!("#{}", self.id.to_string()).serialize(serializer)
        }
    }

    impl<'de, 'r, K, T: 'r, Cx> DeserializeWithCx<'de, Cx> for TagKey<K, T>
    where
        K: FromStr + Clone + 'r,
        <K as FromStr>::Err: std::fmt::Display,
        Cx: LocalContext<&'r Registry<K, T>>,
    {
        fn deserialize_with_cx<D>(
            deserializer: local_cx::WithLocalCx<D, Cx>,
        ) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor<K>(PhantomData<K>);

            impl<K> serde::de::Visitor<'_> for Visitor<K>
            where
                K: FromStr + Clone,
                <K as FromStr>::Err: std::fmt::Display,
            {
                type Value = K;

                fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(formatter, "a string")
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    v.strip_prefix('#')
                        .ok_or_else(|| serde::de::Error::custom("not a tag key"))?
                        .parse::<K>()
                        .map_err(serde::de::Error::custom)
                }
            }

            let registry = deserializer.local_cx.acquire();
            let id = deserializer
                .inner
                .deserialize_str(Visitor(PhantomData::<K>))?;
            Ok(Self {
                registry: registry.key.clone(),
                id,
            })
        }
    }
}
