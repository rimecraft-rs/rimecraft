//! Registry entry types.

use std::{collections::HashSet, ops::Deref};

use parking_lot::RwLock;

use crate::{RegistryCx, key::Key, tag::TagKey};

/// Type holds a value that can be registered
/// in a registry.
#[allow(clippy::exhaustive_enums)]
pub enum Entry<'a, T, Cx>
where
    Cx: RegistryCx,
{
    /// Holds the value directly.
    Direct(T),
    /// Holds the value by reference.
    Ref(&'a RefEntry<T, Cx>),
}

impl<T, Cx> Entry<'_, T, Cx>
where
    Cx: RegistryCx,
{
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
    pub fn key(&self) -> Option<&Key<T, Cx>> {
        match self {
            Entry::Direct(_) => None,
            Entry::Ref(entry) => Some(entry.key()),
        }
    }

    /// Gets the id of this entry.
    ///
    /// Returns `None` if the entry is a direct value.
    #[inline]
    pub fn id(&self) -> Option<&Cx::Id> {
        self.key().map(Key::value)
    }
}

impl<T, Cx> From<T> for Entry<'_, T, Cx>
where
    Cx: RegistryCx,
{
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
pub struct RefEntry<T, Cx>
where
    Cx: RegistryCx,
{
    pub(crate) raw: usize,
    pub(crate) key: Key<T, Cx>,
    pub(crate) value: Option<T>,
    pub(crate) tags: RwLock<HashSet<TagKey<T, Cx>>>,
    pub(crate) is_default: bool,

    #[cfg(feature = "marking-leaked")]
    pub(crate) marker: marking::LeakedPtrMarker,
}

impl<T, Cx> RefEntry<T, Cx>
where
    Cx: RegistryCx,
{
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
    pub fn key(&self) -> &Key<T, Cx> {
        &self.key
    }

    /// Gets the tags of this entry.
    #[inline]
    pub fn tags(&self) -> TagsGuard<'_, T, Cx> {
        TagsGuard {
            inner: self.tags.read(),
        }
    }

    /// Whether this entry is the default entry.
    #[inline]
    pub fn is_default(&self) -> bool {
        self.is_default
    }
}

#[cfg(feature = "marking-leaked")]
impl<T, Cx> RefEntry<T, Cx>
where
    Cx: RegistryCx,
{
    /// Gets the leaked marker of this registry.
    #[inline]
    pub fn marker_leaked(&self) -> marking::LeakedPtrMarker {
        self.marker
    }
}

impl<T, Cx> std::fmt::Debug for Entry<'_, T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: std::fmt::Debug,
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct(arg0) => f.debug_tuple("Direct").field(arg0).finish(),
            Self::Ref(arg0) => f.debug_tuple("Ref").field(arg0).finish(),
        }
    }
}

impl<T, Cx> std::fmt::Debug for RefEntry<T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: std::fmt::Debug,
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefEntry")
            .field("raw", &self.raw)
            .field("key", &self.key)
            .field("value", &self.value)
            .field("tags", &self.tags)
            .finish_non_exhaustive()
    }
}

/// Guard of tags.
pub struct TagsGuard<'a, T, Cx>
where
    Cx: RegistryCx,
{
    inner: parking_lot::RwLockReadGuard<'a, HashSet<TagKey<T, Cx>>>,
}

impl<T, Cx> Deref for TagsGuard<'_, T, Cx>
where
    Cx: RegistryCx,
{
    type Target = HashSet<TagKey<T, Cx>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, Cx> std::fmt::Debug for TagsGuard<'_, T, Cx>
where
    Cx: RegistryCx,
    Cx::Id: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TagsGuard").field(&self.inner).finish()
    }
}

#[cfg(feature = "serde")]
mod serde {
    use std::hash::Hash;

    use local_cx::{LocalContext, serde::DeserializeWithCx};

    use crate::{Query, Reg, Registry, RegistryCx};

    use super::RefEntry;

    impl<T, Cx> serde::Serialize for RefEntry<T, Cx>
    where
        Cx: RegistryCx,
        Cx::Id: serde::Serialize,
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

    impl<'a, 'de, T, Cx, L> DeserializeWithCx<'de, L> for &'a RefEntry<T, Cx>
    where
        Cx: RegistryCx,
        Cx::Id: DeserializeWithCx<'de, L> + Hash + Eq,
        L: LocalContext<&'a Registry<T, Cx>>,
    {
        fn deserialize_with_cx<D>(
            deserializer: local_cx::WithLocalCx<D, L>,
        ) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let cx = deserializer.local_cx;
            let id = Cx::Id::deserialize_with_cx(deserializer)?;
            cx.acquire()
                .get(&Query(&id))
                .map(Reg::to_entry)
                .ok_or_else(|| serde::de::Error::custom("unknown registry key"))
        }
    }
}

#[cfg(feature = "edcode")]
mod edcode {
    use edcode2::{Buf, BufExt as _, BufMut, BufMutExt as _, Decode, Encode};
    use local_cx::{ForwardToWithLocalCx, LocalContext, WithLocalCx};

    use crate::{Reg, Registry, RegistryCx};

    use super::{Entry, RefEntry};

    impl<'r, T: 'r, Cx, Fw> Encode<Fw> for RefEntry<T, Cx>
    where
        Cx: RegistryCx,
        Cx::Id: Clone,
        Fw: ForwardToWithLocalCx<Forwarded: BufMut>,
        Fw::LocalCx: LocalContext<&'r Registry<T, Cx>>,
    {
        fn encode(&self, buf: Fw) -> Result<(), edcode2::BoxedError<'static>> {
            let buf = buf.forward();
            let registry = buf.local_cx.acquire();
            let mut buf = buf.inner;
            let id = Reg::to_raw_id(registry.get(self.key()).ok_or_else(|| {
                edcode2::BoxedError::<'static>::from(format!(
                    "unknown registry id {}",
                    self.key().value()
                ))
            })?);
            buf.put_variable((id + 1) as u32);
            Ok(())
        }
    }

    impl<'a, 'r, 'de, T: 'r, Fw, Cx> Decode<'de, Fw> for &'a RefEntry<T, Cx>
    where
        'r: 'a,
        Cx: RegistryCx,
        Fw: ForwardToWithLocalCx<Forwarded: Buf>,
        Fw::LocalCx: LocalContext<&'r Registry<T, Cx>>,
    {
        fn decode(buf: Fw) -> Result<Self, edcode2::BoxedError<'de>> {
            let mut buf = buf.forward();
            let id = buf.inner.get_variable::<u32>() as usize - 1;
            buf.local_cx
                .acquire()
                .of_raw(id)
                .map(Reg::to_entry)
                .ok_or_else(|| format!("unknown registry id: {id}").into())
        }
    }

    impl<'r, T, Cx, Fw> Encode<Fw> for Entry<'_, T, Cx>
    where
        Cx: RegistryCx,
        Cx::Id: Clone,
        T: Encode<WithLocalCx<Fw::Forwarded, Fw::LocalCx>> + 'r,
        Fw: ForwardToWithLocalCx<Forwarded: BufMut>,
        Fw::LocalCx: LocalContext<&'r Registry<T, Cx>>,
    {
        fn encode(&self, buf: Fw) -> Result<(), edcode2::BoxedError<'static>> {
            let mut buf = buf.forward();
            match self {
                Entry::Direct(value) => {
                    buf.inner.put_variable(0i32);
                    value.encode(buf)
                }
                Entry::Ref(entry) => entry.encode(buf),
            }
        }
    }

    impl<'a, 'r, 'de, T, Cx, Fw> Decode<'de, Fw> for Entry<'a, T, Cx>
    where
        'r: 'a,
        Cx: RegistryCx,
        Cx::Id: Clone,
        T: Decode<'de, WithLocalCx<Fw::Forwarded, Fw::LocalCx>> + 'r,
        Fw: ForwardToWithLocalCx<Forwarded: Buf>,
        Fw::LocalCx: LocalContext<&'r Registry<T, Cx>>,
    {
        fn decode(buf: Fw) -> Result<Self, edcode2::BoxedError<'de>> {
            let mut buf = buf.forward();
            let id = buf.inner.get_variable::<u32>() as usize;
            match id {
                0 => T::decode(buf).map(Entry::Direct),
                id => {
                    let registry = buf.local_cx.acquire();
                    registry
                        .of_raw(id - 1)
                        .map(|r| Entry::Ref(Reg::to_entry(r)))
                        .ok_or_else(|| format!("unknown registry id: {}", id - 1).into())
                }
            }
        }
    }
}
