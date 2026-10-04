//! Registry key related types.

use std::{hash::Hash, marker::PhantomData};

use crate::RegistryCx;

/// A key for a value in a registry in a context
/// where a root registry is available.
pub struct Key<T, Cx>
where
    Cx: RegistryCx,
{
    /// The id of the registry in the root registry.
    registry: Cx::Id,
    /// The id of the value in the registry specified
    /// by [`Self::registry`].
    value: Cx::Id,

    _marker: PhantomData<T>,
}

impl<T, Cx> Key<T, Cx>
where
    Cx: RegistryCx,
{
    /// Creates a new key.
    #[inline]
    pub const fn new(registry: Cx::Id, value: Cx::Id) -> Self {
        Self {
            registry,
            value,
            _marker: PhantomData,
        }
    }

    /// Gets the id of the value in the registry.
    #[inline]
    pub fn value(&self) -> &Cx::Id {
        &self.value
    }

    /// Gets the id of the registry in the root registry.
    #[inline]
    pub fn registry(&self) -> &Cx::Id {
        &self.registry
    }

    #[doc(hidden)]
    #[inline]
    pub fn cast<V>(self) -> Key<V, Cx> {
        Key {
            registry: self.registry,
            value: self.value,
            _marker: PhantomData,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn cast_ref<V>(&self) -> &Key<V, Cx> {
        unsafe { &*std::ptr::from_ref(self).cast::<Key<V, Cx>>() }
    }
}

impl<T, Cx> Key<T, Cx>
where
    Cx: RegistryCx<Id: Root>,
{
    /// Creates a new key with the root registry.
    #[inline]
    pub fn with_root(value: Cx::Id) -> Self {
        Self::new(Cx::Id::root(), value)
    }
}

impl<T, Cx> Hash for Key<T, Cx>
where
    Cx: RegistryCx,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.registry.hash(state);
        self.value.hash(state);
    }
}

impl<T, Cx> Clone for Key<T, Cx>
where
    Cx: RegistryCx<Id: Clone>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            value: self.value.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T, Cx> Copy for Key<T, Cx> where Cx: RegistryCx<Id: Copy> {}

impl<T, Cx> std::fmt::Debug for Key<T, Cx>
where
    Cx: RegistryCx<Id: std::fmt::Debug>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RegistryKey")
            .field(&self.registry)
            .field(&self.value)
            .finish()
    }
}

impl<T, Cx> PartialEq for Key<T, Cx>
where
    Cx: RegistryCx<Id: PartialEq>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.registry == other.registry && self.value == other.value
    }
}

impl<T, Cx> Eq for Key<T, Cx> where Cx: RegistryCx<Id: Eq> {}

impl<T, Cx: RegistryCx> AsRef<Cx::Id> for Key<T, Cx> {
    #[inline]
    fn as_ref(&self) -> &Cx::Id {
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
    use local_cx::{LocalContext, serde::DeserializeWithCx};

    use crate::{Registry, RegistryCx};

    use super::Key;

    impl<T, Cx> serde::Serialize for Key<T, Cx>
    where
        Cx: RegistryCx,
        Cx::Id: serde::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.value.serialize(serializer)
        }
    }

    impl<'r, 'de, T: 'r, L, Cx> DeserializeWithCx<'de, L> for Key<T, Cx>
    where
        L: LocalContext<&'r Registry<T, Cx>>,
        Cx: RegistryCx,
        Cx::Id: DeserializeWithCx<'de, L> + Clone,
    {
        fn deserialize_with_cx<D>(
            deserializer: local_cx::WithLocalCx<D, L>,
        ) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let registry = deserializer.local_cx.acquire();
            let value = Cx::Id::deserialize_with_cx(deserializer)?;
            Ok(Self::new(registry.key.value.clone(), value))
        }
    }
}

/// Helper module for `edcode` support.
#[cfg(feature = "edcode")]
pub mod edcode {

    use edcode2::{Decode, Encode};
    use local_cx::{ForwardToWithLocalCx, LocalContext, WithLocalCx};

    use crate::{Registry, RegistryCx};

    use super::{Key, Root};

    impl<T, Cx, B> Encode<B> for Key<T, Cx>
    where
        Cx: RegistryCx,
        Cx::Id: Encode<B>,
    {
        #[inline]
        fn encode(&self, buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            self.value.encode(buf)
        }
    }

    impl<'r, 'de, T: 'r, Cx, Fw> Decode<'de, Fw> for Key<T, Cx>
    where
        Cx: RegistryCx,
        Cx::Id: Decode<'de, WithLocalCx<Fw::Forwarded, Fw::LocalCx>> + Clone,
        Fw: ForwardToWithLocalCx,
        Fw::LocalCx: LocalContext<&'r Registry<T, Cx>>,
    {
        #[inline]
        fn decode(buf: Fw) -> Result<Self, edcode2::BoxedError<'de>> {
            let buf = buf.forward();
            let registry = buf.local_cx.acquire();
            let value = Cx::Id::decode(buf)?;
            Ok(Self::new(registry.key.value.clone(), value))
        }
    }

    /// Wrapper for registry reference keys.
    #[derive(Debug, Clone, Copy)]
    pub struct RegRef<T>(pub T);

    impl<T, Cx, B> Encode<B> for RegRef<&Key<T, Cx>>
    where
        Cx: RegistryCx,
        Cx::Id: Encode<B>,
    {
        #[inline]
        fn encode(&self, buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            self.0.value.encode(buf)
        }
    }

    impl<T, Cx, B> Encode<B> for RegRef<Key<T, Cx>>
    where
        Cx: RegistryCx,
        Cx::Id: Encode<B>,
    {
        #[inline]
        fn encode(&self, buf: B) -> Result<(), edcode2::BoxedError<'static>> {
            RegRef(&self.0).encode(buf)
        }
    }

    impl<'de, T, Cx, B> Decode<'de, B> for RegRef<Key<T, Cx>>
    where
        Cx: RegistryCx,
        Cx::Id: Decode<'de, B> + Root + Clone,
    {
        #[inline]
        fn decode(buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
            Ok(Self(Key::new(Cx::Id::root(), Cx::Id::decode(buf)?)))
        }
    }
}
