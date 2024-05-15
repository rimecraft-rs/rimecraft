//! Traits for serialization and deserialization of packets.

use std::marker::PhantomData;

use bytes::{Buf, BufMut};
use codecs::Variable;

pub mod codecs;

/// A boxed error type.
pub type BoxedError<'a> = Box<dyn std::error::Error + Send + Sync + 'a>;

/// Packet encoders.
pub trait Encode<B> {
    /// Encodes this packet into the buffer.
    #[allow(clippy::missing_errors_doc)]
    fn encode(&self, buf: B) -> Result<(), BoxedError<'static>>;
}

/// Packet decoders.
pub trait Decode<'de, B>: Sized {
    /// Decodes this packet from the buffer.
    #[allow(clippy::missing_errors_doc)]
    fn decode(buf: B) -> Result<Self, BoxedError<'de>>;
}

/// Packet decoders that decode in place.
pub trait DecodeInPlace<'de, B> {
    /// Decodes this packet from the buffer in place.
    #[allow(clippy::missing_errors_doc)]
    fn decode_in_place(&mut self, buf: B) -> Result<(), BoxedError<'de>>;
}

impl<'de, B, T> DecodeInPlace<'de, B> for T
where
    T: Decode<'de, B>,
{
    #[inline]
    fn decode_in_place(&mut self, buf: B) -> Result<(), BoxedError<'de>> {
        *self = Decode::decode(buf)?;
        Ok(())
    }
}

/// Packet decoders that decodes into a specified type.
pub trait DecodeSeed<'de, B> {
    /// The output type of the decoder.
    type Output;

    /// Decodes this packet from the buffer.
    #[allow(clippy::missing_errors_doc)]
    fn decode(self, buf: B) -> Result<Self::Output, BoxedError<'de>>;
}

impl<'de, B, T> DecodeSeed<'de, B> for PhantomData<T>
where
    T: Decode<'de, B>,
{
    type Output = T;

    #[inline(always)]
    fn decode(self, buf: B) -> Result<Self::Output, BoxedError<'de>> {
        Decode::decode(buf)
    }
}

/// Extension trait for [`BufMut`].
pub trait BufMutExt {
    /// Puts a variable value into the buffer.
    #[inline]
    fn put_variable<'a, T>(&'a mut self, value: T)
    where
        Variable<T>: Encode<&'a mut Self>,
        Self: Sized,
    {
        Variable(value)
            .encode(self)
            .expect("a variable value should be encoded without errors")
    }

    /// Puts a boolean value into the buffer.
    #[inline]
    fn put_bool<'a>(&'a mut self, value: bool)
    where
        bool: Encode<&'a mut Self>,
    {
        value
            .encode(self)
            .expect("a bool value should be encoded without errors")
    }
}

/// Extension trait for [`Buf`].
pub trait BufExt {
    /// Gets a variable value from the buffer.
    #[inline]
    fn get_variable<'a, T>(&'a mut self) -> T
    where
        Variable<T>: Decode<'a, &'a mut Self>,
        Self: Sized,
    {
        Variable::<T>::decode(self)
            .expect("the variable value should not be overflowed")
            .0
    }

    /// Gets a boolean value from the buffer.
    #[inline]
    fn get_bool<'a>(&'a mut self) -> bool
    where
        bool: Decode<'a, &'a mut Self>,
    {
        bool::decode(self).expect("the bool value should not be overflowed")
    }
}

impl<T: BufMut + ?Sized> BufMutExt for T {}
impl<T: Buf + ?Sized> BufExt for T {}

#[cfg(test)]
mod tests;
