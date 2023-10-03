mod imp;

#[cfg(test)]
mod tests;

/// Describes types that can be encoded into a packet buffer.
pub trait Encode {
    /// Encode into a buffer.
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut;
}

/// [`Encode`], but can be used as trait objects.
pub trait BytesEncode {
    fn encode_bytes(&self, bytes: &mut bytes::BytesMut) -> anyhow::Result<()>;
}

impl<T> BytesEncode for T
where
    T: Encode,
{
    #[inline]
    fn encode_bytes(&self, bytes: &mut bytes::BytesMut) -> anyhow::Result<()> {
        self.encode(bytes)
    }
}

/// Describes types that can be decoded from a packet buffer.
pub trait Decode<'de> {
    /// The resulting type.
    type Output;

    /// Decode from a buffer.
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf;
}

/// Represents types that can be updated from a buffer.
pub trait Update: Encode {
    /// Update from a buffer.
    fn update<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf;
}

impl<T> Update for T
where
    T: Encode + for<'de> Decode<'de, Output = T>,
{
    fn update<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        *self = Self::decode(buf)?;
        Ok(())
    }
}

/// Layer for encoding and decoding in nbt binary format for packets.
#[cfg(feature = "nbt")]
pub struct Nbt<'a, T>(pub &'a T);

/// Layer for encoding and decoding in json utf8 for packets.
#[cfg(feature = "json")]
pub struct Json<'a, T>(pub &'a T);

/// Represents a variable integer.
pub struct VarI32(pub i32);

impl VarI32 {
    /// Get the encoded bytes length of this integer.
    pub fn len(self) -> usize {
        for i in 1..5 {
            if (self.0 & -1 << (i * 7)) == 0 {
                return i as usize;
            }
        }

        5
    }

    /// Whether the encoded bytes is empty.
    ///
    /// This should always be `false`.
    #[inline]
    pub fn is_empty(self) -> bool {
        false
    }
}
