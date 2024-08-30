//! Serde support for local context.

#![cfg(feature = "serde")]
#![allow(clippy::missing_errors_doc)]

use std::marker::PhantomData;

use crate::WithLocalCx;

/// Serialize the value with a local context.
pub trait SerializeWithCx<LocalCx> {
    /// Serialize the value with the given serializer and the local context.
    fn serialize_with_cx<S>(&self, serializer: WithLocalCx<S, &LocalCx>) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;
}

/// Deserialize the value with a local context.
pub trait DeserializeWithCx<'de, LocalCx>: Sized {
    /// Deserialize the value with the given deserializer and the local context.
    fn deserialize_with_cx<D>(deserializer: WithLocalCx<D, &LocalCx>) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>;

    /// Deserialize the value in place with the given deserializer and the local context.
    #[inline]
    fn deserialize_in_place_with_cx<D>(
        this: &mut Self,
        deserializer: WithLocalCx<D, &LocalCx>,
    ) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *this = Self::deserialize_with_cx(deserializer)?;
        Ok(())
    }
}

impl<T, Cx> serde::Serialize for WithLocalCx<T, Cx>
where
    T: SerializeWithCx<Cx>,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize_with_cx(WithLocalCx {
            inner: serializer,
            local_cx: &self.local_cx,
        })
    }
}

impl<'de, T, Cx> serde::de::DeserializeSeed<'de> for WithLocalCx<PhantomData<T>, Cx>
where
    T: DeserializeWithCx<'de, Cx>,
{
    type Value = T;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize_with_cx(WithLocalCx {
            inner: deserializer,
            local_cx: &self.local_cx,
        })
    }
}
