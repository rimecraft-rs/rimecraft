pub mod encrypt;
pub mod listener;
pub mod packet;
mod packet_buf_imp;

use crate::prelude::*;

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
/// The `'de` lifetime can be used sometimes, like with serde.
pub trait Decode<'de> {
    /// The resulting type.
    type Output;

    /// Decode from a buffer.
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf;
}

pub trait NetSync: Encode {
    fn read_buf<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf;
}

impl<T> NetSync for T
where
    T: Encode + for<'de> Decode<'de, Output = T>,
{
    fn read_buf<B>(&mut self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::Buf,
    {
        *self = Self::decode(buf)?;
        Ok(())
    }
}

/// Layer for encoding and decoding in nbt binary format for packets.
pub struct Nbt<'a, T>(pub &'a T);

/// Layer for encoding and decoding in json utf8 for packets.
pub struct Json<'a, T>(pub &'a T);

struct ReadAdapt<'a, T: 'a>(pub &'a mut T)
where
    T: bytes::Buf;

impl<T> std::io::Read for ReadAdapt<'_, T>
where
    T: bytes::Buf,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        unsafe { &mut *(self.0 as *mut T as *mut bytes::buf::Reader<T>) }.read(buf)
    }
}

struct WriteAdapt<'a, T: 'a>(pub &'a mut T)
where
    T: bytes::BufMut;

impl<T> std::io::Write for WriteAdapt<'_, T>
where
    T: bytes::BufMut,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe { &mut *(self.0 as *mut T as *mut bytes::buf::Writer<T>) }.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        unsafe { &mut *(self.0 as *mut T as *mut bytes::buf::Writer<T>) }.flush()
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Handshaking = -1,
    Play = 0,
    Status = 1,
    Login = 2,
}

impl State {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            -1 => Some(Self::Handshaking),
            0 => Some(Self::Play),
            1 => Some(Self::Status),
            2 => Some(Self::Login),
            _ => None,
        }
    }
}
