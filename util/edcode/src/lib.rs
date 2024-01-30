pub mod error;
mod imp;

#[cfg(test)]
mod tests;

pub use bytes;

/// Describes types that can be encoded into a packet buffer.
pub trait Encode {
    type Error;

    /// Encode into a buffer.
    fn encode<B>(&self, buf: B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut;
}

type BoxedError = Box<dyn std::error::Error + Send + Sync>;

/// [`Encode`], but can be used as trait objects.
pub trait BytesEncode {
    fn encode_bytes(&self, bytes: &mut bytes::BytesMut) -> Result<(), BoxedError>;
}

impl<T> BytesEncode for T
where
    T: Encode,
    <T as Encode>::Error: std::error::Error + Send + Sync + 'static,
{
    #[inline]
    fn encode_bytes(&self, bytes: &mut bytes::BytesMut) -> Result<(), BoxedError> {
        self.encode(bytes).map_err(From::from)
    }
}

/// Describes types that can be decoded from a packet buffer.
pub trait Decode {
    /// The resulting type.
    type Output;

    type Error;

    /// Decode from a buffer.
    fn decode<B>(buf: B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf;
}

/// Represents types that can be updated from a buffer.
pub trait Update: Encode {
    type Error;

    /// Update from a buffer.
    fn update<B>(&mut self, buf: B) -> Result<(), <Self as Update>::Error>
    where
        B: bytes::Buf;
}

impl<T, E> Update for T
where
    T: Encode<Error = E> + Decode<Output = T, Error = E>,
{
    type Error = E;

    #[inline]
    fn update<B>(&mut self, buf: B) -> Result<(), <Self as Update>::Error>
    where
        B: bytes::Buf,
    {
        *self = Self::decode(buf)?;
        Ok(())
    }
}

/// Layer for encoding and decoding in nbt binary format for packets.
#[cfg(feature = "nbt")]
pub struct Nbt<T>(pub T);

/// Layer for encoding and decoding in json utf8 for packets.
#[cfg(feature = "json")]
pub struct Json<T>(pub T);

/// Represents a variable integer.
pub struct VarI32(pub i32);

impl VarI32 {
    /// Get the encoded bytes length of this integer.
    #[inline]
    pub fn len(self) -> usize {
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
