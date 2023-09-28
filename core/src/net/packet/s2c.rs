use crate::{
    net::{listener, Decode, Encode},
    text::Text,
};

pub struct LoginCompression {
    threshold: i32,
}

impl LoginCompression {
    #[inline]
    pub fn new(threshold: i32) -> Self {
        Self { threshold }
    }

    pub fn threshold(&self) -> i32 {
        self.threshold
    }
}

impl Encode for LoginCompression {
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        crate::VarInt(self.threshold).encode(buf)
    }
}

impl<'de> Decode<'de> for LoginCompression {
    type Output = Self;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(Self {
            threshold: crate::VarInt::decode(buf)?,
        })
    }
}

impl<L> super::Packet<L> for LoginCompression where L: listener::Accept<Self> {}

pub struct LoginDisconnect<T>
where
    T: Text,
{
    reason: T,
}

impl<T> LoginDisconnect<T> where T: Text {
    #[inline]
    pub fn new(reason: T) -> Self {
        Self { reason }
    }
}


