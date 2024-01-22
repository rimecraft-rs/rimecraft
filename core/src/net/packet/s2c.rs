use std::{convert::Infallible, string::FromUtf8Error};

use bytes::Bytes;
use rimecraft_edcode::{
    error::{ErrorWithVarI32Err, VarI32TooBigError},
    Decode, Encode, VarI32,
};
use rimecraft_primitives::Id;
use rsa::{pkcs8::DecodePublicKey, RsaPublicKey};

use crate::{net::listener, text::Text, BoxedError};

use super::error::PayloadTooLargeError;

#[derive(Debug)]
pub struct LoginCompression {
    threshold: i32,
}

impl LoginCompression {
    #[inline]
    pub fn new(threshold: i32) -> Self {
        Self { threshold }
    }

    #[inline]
    pub fn threshold(&self) -> i32 {
        self.threshold
    }
}

impl Encode for LoginCompression {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        VarI32(self.threshold).encode(buf)
    }
}

impl<'de> Decode<'de> for LoginCompression {
    type Output = Self;

    type Error = VarI32TooBigError;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        Ok(Self {
            threshold: VarI32::decode(buf)?,
        })
    }
}

impl<L> super::Packet<L> for LoginCompression where L: listener::Accept<Self> {}

#[derive(Debug)]
pub struct LoginDisconnect {
    reason: Text,
}

impl LoginDisconnect {
    #[inline]
    pub fn new(reason: Text) -> Self {
        Self { reason }
    }

    #[inline]
    pub fn reason(&self) -> &Text {
        &self.reason
    }
}

impl Encode for LoginDisconnect {
    type Error = Infallible;

    fn encode<B>(&self, _buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        unimplemented!("text ser")
    }
}

impl<'de> Decode<'de> for LoginDisconnect {
    type Output = Self;

    type Error = Infallible;

    fn decode<B>(_buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        unimplemented!("text deser")
    }
}

impl<L> super::Packet<L> for LoginDisconnect where L: listener::Accept<Self> {}

#[derive(Debug)]
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
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
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

    type Error = ErrorWithVarI32Err<FromUtf8Error>;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
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

#[derive(Debug)]
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
    pub fn channel_id(&self) -> &Id {
        &self.channel
    }
}

impl Encode for LoginQueryReq {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        VarI32(self.query_id).encode(buf)?;
        self.channel.encode(buf)?;
        buf.put_slice(&self.payload[..]);
        Ok(())
    }
}

impl<'de> Decode<'de> for LoginQueryReq {
    type Output = Self;

    type Error = BoxedError;

    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let query_id = VarI32::decode(buf)?;
        let channel = Id::decode(buf)?;

        let remaining = buf.remaining();
        if remaining <= super::QUERY_MAX_PAYLOAD_LEN {
            // this was changed in 1.20.2
            buf.advance(remaining);
            let payload = Bytes::new();
            Ok(Self {
                query_id,
                channel,
                payload,
            })
        } else {
            Err(PayloadTooLargeError {
                max: super::QUERY_MAX_PAYLOAD_LEN,
            }
            .into())
        }
    }
}

impl<L> super::Packet<L> for LoginQueryReq where L: listener::Accept<Self> {}

//TODO: LoginSuccessS2CPacket and authlib's GameProfile implementation

#[derive(Debug)]
pub struct PingResult {
    start_time: u64,
}

impl PingResult {
    #[inline]
    pub fn new(start_time: u64) -> Self {
        Self { start_time }
    }

    #[inline]
    pub fn start_time(&self) -> u64 {
        self.start_time
    }
}

impl Encode for PingResult {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        buf.put_i64(self.start_time as i64);
        Ok(())
    }
}

impl<'de> Decode<'de> for PingResult {
    type Output = Self;

    type Error = Infallible;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        Ok(Self {
            start_time: buf.get_i64() as u64,
        })
    }
}

impl<L> super::Packet<L> for PingResult where L: listener::Accept<Self> {}

//TODO: QueryResponseS2CPacket and ServerMetadata
