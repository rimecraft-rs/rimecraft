//! Commonly used data container codecs.

use std::marker::PhantomData;

use crate::WithLocalCx;

use super::DeserializeWithCx;

/// Seed for deserializing an [`Option`] with local context.
#[derive(Debug)]
pub struct OptionSeed<T, Cx>(pub PhantomData<T>, pub Cx);

impl<'de, T, Cx> serde::de::DeserializeSeed<'de> for OptionSeed<T, Cx>
where
    T: DeserializeWithCx<'de, Cx>,
{
    type Value = Option<T>;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_option(self)
    }
}

impl<'de, T, Cx> serde::de::Visitor<'de> for OptionSeed<T, Cx>
where
    T: DeserializeWithCx<'de, Cx>,
{
    type Value = Option<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "an optional value")
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize_with_cx(WithLocalCx {
            inner: deserializer,
            local_cx: self.1,
        })
        .map(Some)
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }
}
