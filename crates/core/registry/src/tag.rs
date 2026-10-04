//! Tag related types.

use std::{collections::HashMap, hash::Hash};

use crate::{Registry, RegistryCx, key::Key};

/// Key of a tag.
pub struct TagKey<T, Cx>
where
    Cx: RegistryCx,
{
    /// The registry reference.
    pub registry: Key<Registry<T, Cx>, Cx>,
    /// The tag id.
    pub id: Cx::Id,
}

impl<T, Cx> Hash for TagKey<T, Cx>
where
    Cx: RegistryCx,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.registry.hash(state);
        self.id.hash(state);
    }
}

impl<T, Cx> PartialEq for TagKey<T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.registry == other.registry && self.id == other.id
    }
}

impl<T, Cx> Eq for TagKey<T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: Eq,
{
}

impl<T, Cx> Clone for TagKey<T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            id: self.id.clone(),
        }
    }
}

impl<T, Cx> Copy for TagKey<T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: Copy,
{
}

impl<T, Cx> std::fmt::Debug for TagKey<T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TagKey")
            .field(&self.registry.value())
            .field(&self.id)
            .finish()
    }
}

/// Tags of a registry.
pub struct Tags<'r, T, Cx>
where
    Cx: RegistryCx,
{
    pub(crate) inner: parking_lot::RwLockReadGuard<'r, HashMap<TagKey<T, Cx>, Vec<usize>>>,
    pub(crate) registry: &'r Registry<T, Cx>,
}

impl<T, Cx> Tags<'_, T, Cx>
where
    Cx: RegistryCx,
{
    /// Gets an iterator over the tags.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T, Cx>
    where
        Cx: RegistryCx,
    {
        Iter {
            inner: self.inner.iter(),
            registry: self.registry,
        }
    }
}

impl<T, Cx> std::fmt::Debug for Tags<'_, T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tags")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl<'a: 'a, T, Cx> IntoIterator for &'a Tags<'_, T, Cx>
where
    Cx: RegistryCx,
{
    type Item = (&'a TagKey<T, Cx>, crate::Entries<'a, T, Cx>);

    type IntoIter = Iter<'a, T, Cx>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator of tags.
pub struct Iter<'a, T, Cx>
where
    Cx: RegistryCx,
{
    inner: std::collections::hash_map::Iter<'a, TagKey<T, Cx>, Vec<usize>>,
    registry: &'a Registry<T, Cx>,
}

impl<'a, T, Cx> Iterator for Iter<'a, T, Cx>
where
    Cx: RegistryCx,
{
    type Item = (&'a TagKey<T, Cx>, crate::Entries<'a, T, Cx>);

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

impl<T, Cx> std::fmt::Debug for Iter<'_, T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Iter")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

/// Helper module for `serde` support.
#[cfg(feature = "serde")]
pub mod serde {
    use std::{marker::PhantomData, str::FromStr};

    use local_cx::{LocalContext, serde::DeserializeWithCx};

    use crate::{Registry, RegistryCx};

    use super::TagKey;

    /// `TagKey` serialize and deserailize wrapper
    /// without `#` prefix.
    #[derive(Debug, Clone, Copy)]
    pub struct Unprefixed<T>(pub T);

    impl<T, Cx> serde::Serialize for Unprefixed<&TagKey<T, Cx>>
    where
        Cx: RegistryCx,
        Cx::Id: serde::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.id.serialize(serializer)
        }
    }

    impl<T, Cx> serde::Serialize for Unprefixed<TagKey<T, Cx>>
    where
        Cx: RegistryCx,
        Cx::Id: serde::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Unprefixed(&self.0).serialize(serializer)
        }
    }

    impl<'de, 'r, T: 'r, Cx, L> DeserializeWithCx<'de, L> for Unprefixed<TagKey<T, Cx>>
    where
        Cx: RegistryCx,
        Cx::Id: DeserializeWithCx<'de, L> + Clone,
        L: LocalContext<&'r Registry<T, Cx>>,
    {
        fn deserialize_with_cx<D>(
            deserializer: local_cx::WithLocalCx<D, L>,
        ) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let registry = deserializer.local_cx.acquire();
            Ok(Self(TagKey {
                registry: registry.key.clone(),
                id: Cx::Id::deserialize_with_cx(deserializer)?,
            }))
        }
    }

    impl<T, Cx> serde::Serialize for TagKey<T, Cx>
    where
        Cx: RegistryCx,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            format!("#{}", self.id).serialize(serializer)
        }
    }

    impl<'de, 'r, T: 'r, Cx, L> DeserializeWithCx<'de, L> for TagKey<T, Cx>
    where
        Cx: RegistryCx,
        Cx::Id: FromStr + Clone,
        <Cx::Id as FromStr>::Err: std::fmt::Display,
        L: LocalContext<&'r Registry<T, Cx>>,
    {
        fn deserialize_with_cx<D>(
            deserializer: local_cx::WithLocalCx<D, L>,
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
                .deserialize_str(Visitor(PhantomData::<Cx::Id>))?;
            Ok(Self {
                registry: registry.key.clone(),
                id,
            })
        }
    }
}
