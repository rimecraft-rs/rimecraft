//! Encoding and decoding utilities for packet buffers.

mod imp;

#[cfg(test)]
mod tests;

use std::{borrow::Cow, io};

pub use bytes;

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

/// [`Encode`], but can be used as trait objects.
pub trait BytesEncode {
    /// Encodes into a bytes buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the encoding failed.
    fn encode_bytes(&self, bytes: &mut bytes::BytesMut) -> Result<(), io::Error>;
}

impl<T> BytesEncode for T
where
    T: Encode,
{
    #[inline(always)]
    fn encode_bytes(&self, bytes: &mut bytes::BytesMut) -> Result<(), io::Error> {
        self.encode(bytes)
    }
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

impl<T> Update for T
where
    T: Decode,
{
    #[inline]
    fn update<B>(&mut self, buf: B) -> Result<(), io::Error>
    where
        B: bytes::Buf,
    {
        *self = Self::decode(buf)?;
        Ok(())
    }
}

/// Layer for encoding and decoding in nbt binary format for packets.
#[cfg(feature = "fastnbt")]
#[derive(Debug)]
pub struct Nbt<T>(pub T);

/// Layer for encoding and decoding in json utf8 for packets.
#[cfg(feature = "json")]
#[derive(Debug)]
pub struct Json<T>(pub T);

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

pub fn decode_cow_str<'a, B>(mut buf: &'a mut B) -> io::Result<Cow<'a, str>>
where
    B: bytes::Buf,
{
    let len = VarI32::decode(&mut buf)?.0 as usize;
    if len <= buf.remaining() {
        buf.advance(len);
        std::str::from_utf8(&bytes::Buf::chunk(buf)[..len])
            .map(Cow::Borrowed)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    } else {
        String::decode(buf).map(Cow::Owned)
    }
}
