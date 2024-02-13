//! Serde human-readable controlling layers.

use serde::{Deserializer, Serializer};

/// Wrapper to a [`Serializer`] or [`Deserializer`] that supports
/// configuring `is_human_readable` manually.
#[derive(Debug)]
pub struct HumanReadableControlled<T> {
    inner: T,
    human_readable: bool,
}

impl<T> HumanReadableControlled<T> {
    /// Creates a new `HumanReadableControlled` with the given inner value and human-readable flag.
    #[inline]
    pub const fn new(inner: T, human_readable: bool) -> Self {
        Self {
            inner,
            human_readable,
        }
    }

    /// Returns the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }
}

macro_rules! ser {
    ($($f:ident, $t:ty),*$(,)?) => {
        $(
            #[inline]
            fn $f(self, v: $t) -> Result<Self::Ok, Self::Error> {
                self.inner.$f(v)
            }
        )*
    };
}

macro_rules! ser_gat {
    ($($t:ident),*$(,)?) => {
        $(type $t = <S as Serializer>::$t;)*
    };
}

impl<S> Serializer for HumanReadableControlled<S>
where
    S: Serializer,
{
    ser_gat! {
        Ok, Error,
        SerializeSeq,
        SerializeTuple,
        SerializeTupleStruct,
        SerializeTupleVariant,
        SerializeMap,
        SerializeStruct,
        SerializeStructVariant,
    }

    ser! {
        serialize_bool, bool,
        serialize_i8, i8,
        serialize_i16, i16,
        serialize_i32, i32,
        serialize_i64, i64,
        serialize_i128, i128,
        serialize_u8, u8,
        serialize_u16, u16,
        serialize_u32, u32,
        serialize_u64, u64,
        serialize_u128, u128,
        serialize_f32, f32,
        serialize_f64, f64,
        serialize_char, char,
        serialize_str, &str,
        serialize_bytes, &[u8],
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_none()
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        self.inner.serialize_some(value)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit()
    }

    #[inline]
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit_struct(name)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.inner
            .serialize_unit_variant(name, variant_index, variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        self.inner.serialize_newtype_struct(name, value)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        self.inner
            .serialize_newtype_variant(name, variant_index, variant, value)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.inner.serialize_seq(len)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.inner.serialize_tuple(len)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.inner.serialize_tuple_struct(name, len)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.inner
            .serialize_tuple_variant(name, variant_index, variant, len)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.inner.serialize_map(len)
    }

    #[inline]
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.inner.serialize_struct(name, len)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.inner
            .serialize_struct_variant(name, variant_index, variant, len)
    }

    #[inline]
    fn collect_seq<I>(self, iter: I) -> Result<Self::Ok, Self::Error>
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: serde::Serialize,
    {
        self.inner.collect_seq(iter)
    }

    #[inline]
    fn collect_map<K, V, I>(self, iter: I) -> Result<Self::Ok, Self::Error>
    where
        K: serde::Serialize,
        V: serde::Serialize,
        I: IntoIterator<Item = (K, V)>,
    {
        self.inner.collect_map(iter)
    }

    #[inline]
    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: std::fmt::Display,
    {
        self.inner.collect_str(value)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        self.human_readable
    }
}

macro_rules! deser {
    ($($t:ident),*$(,)?) => {
        $(
            #[inline]
            fn $t<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                self.inner.$t(visitor)
            }
        )*
    };
}

impl<'de, S> Deserializer<'de> for HumanReadableControlled<S>
where
    S: Deserializer<'de>,
{
    type Error = <S as Deserializer<'de>>::Error;

    deser! {
        deserialize_any,
        deserialize_bool,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_i128,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_u128,
        deserialize_f32,
        deserialize_f64,
        deserialize_char,
        deserialize_str,
        deserialize_string,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_option,
        deserialize_unit,
        deserialize_seq,
        deserialize_map,
        deserialize_identifier,
        deserialize_ignored_any,
    }

    #[inline]
    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.inner.deserialize_unit_struct(name, visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.inner.deserialize_newtype_struct(name, visitor)
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.inner.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.inner.deserialize_tuple_struct(name, len, visitor)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.inner.deserialize_struct(name, fields, visitor)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.inner.deserialize_enum(name, variants, visitor)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        self.human_readable
    }
}
