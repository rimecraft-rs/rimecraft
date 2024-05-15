//! Traits for serialization and deserialization of packets.

use std::marker::PhantomData;

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
