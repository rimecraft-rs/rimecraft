use std::convert::Infallible;

use bytes::Bytes;
use rimecraft_edcode::{
    error::{ErrorWithVarI32Err, VarI32TooBigError},
    Decode, Encode, VarI32,
};
use rsa::RsaPrivateKey;

use crate::{net::listener, BoxedError};

use super::error::*;

#[derive(Debug)]
pub struct Handshake {
    proto_ver: i32,
    addr: String,
    port: u16,
    intended_state: ConnectionIntent,
}

impl Handshake {
    #[inline]
    pub fn new(addr: String, port: u16, intended_state: ConnectionIntent) -> Self {
        ///TODO: Need to implement net.minecraft.SharedConstants
        const PROTO_VER: i32 = 114514;

        Self {
            proto_ver: PROTO_VER,
            addr,
            port,
            intended_state,
        }
    }

    #[inline]
    pub fn proto_version(&self) -> i32 {
        self.proto_ver
    }

    #[inline]
    pub fn addr(&self) -> &str {
        &self.addr
    }

    #[inline]
    pub fn port(&self) -> u16 {
        self.port
    }

    #[inline]
    pub fn intended_state(&self) -> ConnectionIntent {
        self.intended_state
    }
}

impl Encode for Handshake {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        VarI32(self.proto_ver).encode(buf)?;
        self.addr.encode(buf)?;
        self.port.encode(buf)?;
        VarI32(self.intended_state as i32).encode(buf)
    }
}

impl<'de> Decode<'de> for Handshake {
    type Output = Self;

    type Error = BoxedError;

    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let proto_ver = VarI32::decode(buf)?;
        let addr = String::decode(buf)?;
        let port = i16::decode(buf).unwrap() as u16;
        let state = VarI32::decode(buf)?;

        Ok(Self {
            proto_ver,
            addr,
            port,
            intended_state: ConnectionIntent::n(state)
                .ok_or(UnknownConnectionIntentError(state))?,
        })
    }
}

impl<L> super::Packet<L> for Handshake
where
    L: listener::Accept<Self>,
{
    #[inline]
    fn new_net_state(&self) -> Option<crate::net::State> {
        Some(self.intended_state.state())
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, enumn::N, Debug)]
pub enum ConnectionIntent {
    Status,
    Login,
}

impl ConnectionIntent {
    #[inline]
    pub fn state(self) -> crate::net::State {
        match self {
            ConnectionIntent::Status => crate::net::State::Status,
            ConnectionIntent::Login => crate::net::State::Login,
        }
    }
}

#[derive(Debug)]
pub struct LoginHello {
    name: String,
    profile_id: uuid::Uuid,
}

impl Encode for LoginHello {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Infallible>
    where
        B: bytes::BufMut,
    {
        self.name.encode(buf)?;
        self.profile_id.encode(buf)
    }
}

impl<'de> Decode<'de> for LoginHello {
    type Output = Self;

    type Error = BoxedError;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let name = String::decode(buf)?;
        let uuid = uuid::Uuid::decode(buf).unwrap();

        Ok(Self {
            name,
            profile_id: uuid,
        })
    }
}

impl<L> super::Packet<L> for LoginHello where L: listener::Accept<Self> {}

#[derive(Debug)]
pub struct LoginQueryRes {
    query_id: i32,
    res: Option<bytes::Bytes>,
}

impl LoginQueryRes {
    #[inline]
    pub fn query_id(&self) -> i32 {
        self.query_id
    }
}

impl Encode for LoginQueryRes {
    type Error = Infallible;

    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        VarI32(self.query_id).encode(buf)?;

        if let Some(ref value) = self.res {
            true.encode(buf)?;
            buf.put_slice(&value[..]);
            Ok(())
        } else {
            false.encode(buf)
        }
    }
}

impl<'de> Decode<'de> for LoginQueryRes {
    type Output = Self;

    type Error = ErrorWithVarI32Err<PayloadTooLargeError>;

    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        struct NullableRes;

        impl<'de> Decode<'de> for NullableRes {
            type Output = bytes::Bytes;

            type Error = PayloadTooLargeError;

            #[inline]
            fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
            where
                B: bytes::Buf,
            {
                let remaining = buf.remaining();
                if remaining <= super::QUERY_MAX_PAYLOAD_LEN {
                    // this was changed in 1.20.2 so the bytes are empty
                    buf.advance(remaining);
                    Ok(Bytes::new())
                } else {
                    Err(PayloadTooLargeError {
                        max: super::QUERY_MAX_PAYLOAD_LEN,
                    })
                }
            }
        }

        let qid = VarI32::decode(buf)?;
        let res = Option::<NullableRes>::decode(buf).map_err(ErrorWithVarI32Err::Target)?;

        Ok(Self { query_id: qid, res })
    }
}

impl<L> super::Packet<L> for LoginQueryRes where L: listener::Accept<Self> {}

#[derive(Debug)]
pub struct LoginKey {
    encrypted_secret_key: Bytes,

    /// The nonce value.
    ///
    /// This value is either encrypted (the left side of `Either`) or signed
    /// (the right side). If encrypted, then it must be done so using the server's public key
    /// and the server verifies it by decrypting and comparing nonces. If signed, then it must
    /// be done so using the user's private key provided from Mojang's server, and the server
    /// verifies by checking if the reconstructed data can be verified using the public key.
    nonce: Bytes,
}

impl Encode for LoginKey {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        self.encrypted_secret_key.encode(buf)?;
        self.nonce.encode(buf)
    }
}

impl<'de> Decode<'de> for LoginKey {
    type Output = LoginKey;

    type Error = VarI32TooBigError;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let encrypted_secret_key = Bytes::decode(buf)?;
        let nonce = Bytes::decode(buf)?;

        Ok(LoginKey {
            encrypted_secret_key,
            nonce,
        })
    }
}

impl<L> super::Packet<L> for LoginKey where L: listener::Accept<Self> {}

impl LoginKey {
    pub fn new<R>(
        rng: &mut R,
        secret_key: &[u8],
        public_key: &rsa::RsaPublicKey,
        nonce: &[u8],
    ) -> Result<LoginKey, rsa::Error>
    where
        R: rsa::rand_core::CryptoRngCore,
    {
        Ok(Self {
            encrypted_secret_key: public_key
                .encrypt(rng, rsa::pkcs1v15::Pkcs1v15Encrypt, secret_key)?
                .into(),
            nonce: public_key
                .encrypt(rng, rsa::pkcs1v15::Pkcs1v15Encrypt, nonce)?
                .into(),
        })
    }

    #[inline]
    pub fn decrypt_secret_key(&self, key: &RsaPrivateKey) -> rsa::Result<bytes::Bytes> {
        key.decrypt(rsa::pkcs1v15::Pkcs1v15Encrypt, &self.encrypted_secret_key)
            .map(From::from)
    }

    #[inline]
    pub fn verify_signed_nonce(&self, nonce: &[u8], private_key: &RsaPrivateKey) -> bool {
        private_key
            .decrypt(rsa::pkcs1v15::Pkcs1v15Encrypt, &self.nonce)
            .map_or(false, |value| nonce == value)
    }
}

#[derive(Debug)]
pub struct QueryPing {
    start_time: u64,
}

impl QueryPing {
    pub fn start_time(&self) -> u64 {
        self.start_time
    }
}

impl Encode for QueryPing {
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

impl<'de> Decode<'de> for QueryPing {
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

impl<L> super::Packet<L> for QueryPing where L: listener::Accept<Self> {}

#[derive(Debug)]
pub struct QueryReq;

impl Encode for QueryReq {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, _buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        Ok(())
    }
}

impl<'de> Decode<'de> for QueryReq {
    type Output = Self;

    type Error = Infallible;

    #[inline]
    fn decode<B>(_buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        Ok(Self)
    }
}

impl<L> super::Packet<L> for QueryReq where L: listener::Accept<Self> {}
