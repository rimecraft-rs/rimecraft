//! Encoding and decoding utilities for packet buffers.

#![deprecated = "use the `rimecraft-packet-codec` crate instead"]

mod imp;

#[cfg(test)]
mod tests;

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

impl<T> Update for T
where
    T: Decode,
{
    #[inline]
    fn update<B>(&mut self, buf: B) -> Result<(), io::Error>
    where
        B: bytes::Buf,
    {
        self.decode_in_place(buf)
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
