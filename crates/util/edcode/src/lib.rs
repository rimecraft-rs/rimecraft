//! Encoding and decoding utilities for packet buffers.

#![deprecated = "use the `edcode2` crate instead"]

use std::io;

pub use bytes;

#[cfg(feature = "derive")]
pub use rimecraft_edcode_derive::{Decode, Encode};

/// Describes types that can be encoded into a packet buffer.
pub trait Encode {
    /// Encode into a buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the encoding failed.
    fn encode<B>(&self, buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut;
}

/// Describes types that can be decoded from a packet buffer.
pub trait Decode: Sized {
    /// Decode from a buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the decoding failed.
    fn decode<B>(buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf;

    /// Decode from a buffer in place.
    ///
    /// # Errors
    ///
    /// Returns an error if the decoding failed.
    #[inline]
    fn decode_in_place<B>(&mut self, buf: B) -> Result<(), io::Error>
    where
        B: bytes::Buf,
    {
        *self = Self::decode(buf)?;
        Ok(())
    }
}

/// Represents types that can be updated from a buffer.
pub trait Update {
    /// Update from a buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the decoding in place failed.
    fn update<B>(&mut self, buf: B) -> Result<(), io::Error>
    where
        B: bytes::Buf;
}

impl<T> Encode for T
where
    T: for<'a> edcode2::Encode<&'a mut dyn bytes::BufMut> + ?Sized,
{
    fn encode<B>(&self, mut buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        edcode2::Encode::encode(self, &mut buf)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}

impl<T> Decode for T
where
    T: for<'a> edcode2::Decode<'static, &'a mut dyn bytes::Buf>,
{
    fn decode<B>(mut buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        let val: &mut dyn bytes::Buf = &mut buf;
        edcode2::Decode::decode(val).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}

impl<T> Update for T
where
    T: for<'a> edcode2::DecodeInPlace<'static, &'a mut dyn bytes::Buf> + ?Sized,
{
    fn update<B>(&mut self, mut buf: B) -> Result<(), io::Error>
    where
        B: bytes::Buf,
    {
        let val: &mut dyn bytes::Buf = &mut buf;
        edcode2::DecodeInPlace::decode_in_place(self, val)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}

/// Represents a variable integer.
#[derive(Debug)]
#[repr(transparent)]
#[doc(alias = "VarInt")]
pub struct VarI32(pub i32);

impl VarI32 {
    /// Get the encoded bytes length of this integer.
    #[inline]
    pub fn encoded_len(self) -> usize {
        (1..5).find(|i| self.0 & -1 << (i * 7) == 0).unwrap_or(5)
    }

    /// Whether the encoded bytes is empty.
    ///
    /// This should always be `false`.
    #[inline]
    pub fn is_empty(self) -> bool {
        false
    }
}

impl<B: bytes::BufMut> edcode2::Encode<B> for VarI32 {
    fn encode(&self, buf: B) -> Result<(), edcode2::BoxedError<'static>> {
        edcode2::Encode::encode(&edcode2::Variable(self.0), buf)
    }
}

impl<'de, B: bytes::Buf> edcode2::Decode<'de, B> for VarI32 {
    fn decode(buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
        <edcode2::Variable<i32> as edcode2::Decode<'de, B>>::decode(buf)
            .map(|edcode2::Variable(i)| Self(i))
    }
}
