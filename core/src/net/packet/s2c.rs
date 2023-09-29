use bytes::Bytes;
use rsa::{pkcs8::DecodePublicKey, RsaPublicKey};

use crate::{
    net::{listener, Decode, Encode},
    text::Text,
    Id,
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

pub struct LoginDisconnect<T> {
    reason: T,
}

impl<T> LoginDisconnect<T> {
    #[inline]
    pub fn new(reason: T) -> Self {
        Self { reason }
    }

    #[inline]
    pub fn reason(&self) -> &T {
        &self.reason
    }
}

impl<T> Encode for LoginDisconnect<T>
where
    T: Text,
{
    fn encode<B>(&self, _buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        unimplemented!("text ser")
    }
}

impl<'de, T> Decode<'de> for LoginDisconnect<T>
where
    T: Text,
{
    type Output = Self;

    fn decode<B>(_buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        unimplemented!("text deser")
    }
}

impl<L, T> super::Packet<L> for LoginDisconnect<T>
where
    L: listener::Accept<Self>,
    T: Text,
{
}

pub struct LoginHello {
    server_id: String,
    pub_key: Bytes,
    nonce: Bytes,
}

impl LoginHello {
    #[inline]
    pub fn new(server_id: String, pub_key: Bytes, nonce: Bytes) -> Self {
        Self {
            server_id,
            pub_key,
            nonce,
        }
    }

    #[inline]
    pub fn server_id(&self) -> &str {
        &self.server_id
    }

    #[inline]
    pub fn pub_key(&self) -> rsa::Result<RsaPublicKey> {
        RsaPublicKey::from_public_key_der(&self.pub_key)
            .map_err(|err| rsa::Error::Pkcs8(rsa::pkcs8::Error::PublicKey(err)))
    }

    #[inline]
    pub fn nonce(&self) -> &[u8] {
        &self.nonce
    }
}

impl Encode for LoginHello {
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        self.server_id.encode(buf)?;
        self.pub_key.encode(buf)?;
        self.nonce.encode(buf)
    }
}

impl<'de> Decode<'de> for LoginHello {
    type Output = Self;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        let server_id = String::decode(buf)?;
        let pub_key = Bytes::decode(buf)?;
        let nonce = Bytes::decode(buf)?;

        Ok(Self {
            server_id,
            pub_key,
            nonce,
        })
    }
}

impl<L> super::Packet<L> for LoginHello where L: listener::Accept<Self> {}

pub struct LoginQueryReq {
    query_id: i32,
    channel: Id,
    payload: Bytes,
}

impl LoginQueryReq {
    #[inline]
    pub fn new(query_id: i32, channel: Id, payload: Bytes) -> Self {
        Self {
            query_id,
            channel,
            payload,
        }
    }

    #[inline]
    pub fn query_id(&self) -> i32 {
        self.query_id
    }

    #[inline]
    pub fn channel(&self) -> &Id {
        &self.channel
    }

    #[inline]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

impl Encode for LoginQueryReq {
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        crate::VarInt(self.query_id).encode(buf)?;
        self.channel.encode(buf)?;
        buf.put_slice(&self.payload[..]);
        Ok(())
    }
}

impl<'de> Decode<'de> for LoginQueryReq {
    type Output = Self;

    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        let query_id = crate::VarInt::decode(buf)?;
        let channel = Id::decode(buf)?;

        let remaining = buf.remaining();
        if remaining <= super::QUERY_MAX_PAYLOAD_LEN {
            let payload = buf.copy_to_bytes(remaining);
            Ok(Self {
                query_id,
                channel,
                payload,
            })
        } else {
            Err(anyhow::anyhow!(
                "payload may not be larger than {} bytes",
                super::QUERY_MAX_PAYLOAD_LEN
            ))
        }
    }
}

impl<L> super::Packet<L> for LoginQueryReq where L: listener::Accept<Self> {}

//TODO: LoginSuccessS2CPacket and authlib's GameProfile implementation

pub struct QueryPong {
    start_time: u64,
}

impl QueryPong {
    #[inline]
    pub fn new(start_time: u64) -> Self {
        Self { start_time }
    }

    #[inline]
    pub fn start_time(&self) -> u64 {
        self.start_time
    }
}

impl Encode for QueryPong {
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        buf.put_i64(self.start_time as i64);
        Ok(())
    }
}

impl<'de> Decode<'de> for QueryPong {
    type Output = Self;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(Self {
            start_time: buf.get_i64() as u64,
        })
    }
}

impl<L> super::Packet<L> for QueryPong where L: listener::Accept<Self> {}

//TODO: QueryResponseS2CPacket and ServerMetadata
