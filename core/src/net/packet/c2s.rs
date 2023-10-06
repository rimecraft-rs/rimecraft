use anyhow::Ok;

use bytes::Bytes;
use rimecraft_edcode::{Decode, Encode, VarI32};
use rsa::RsaPrivateKey;

use crate::net::listener;

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
    pub fn address(&self) -> &str {
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
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
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

    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        let proto_ver = VarI32::decode(buf)?;
        let addr = String::decode(buf)?;
        let port = i16::decode(buf)? as u16;
        let state = VarI32::decode(buf)?;

        Ok(Self {
            proto_ver,
            addr,
            port,
            intended_state: ConnectionIntent::n(state)
                .ok_or_else(|| anyhow::anyhow!("unknown connection intent: {state}"))?,
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
#[derive(Clone, Copy, PartialEq, Eq, enumn::N)]
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

pub struct LoginHello {
    name: String,
    profile_id: uuid::Uuid,
}

impl Encode for LoginHello {
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        self.name.encode(buf)?;
        self.profile_id.encode(buf)
    }
}

impl<'de> Decode<'de> for LoginHello {
    type Output = Self;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        let name = String::decode(buf)?;
        let uuid = uuid::Uuid::decode(buf)?;

        Ok(Self {
            name,
            profile_id: uuid,
        })
    }
}

impl<L> super::Packet<L> for LoginHello where L: listener::Accept<Self> {}

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
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
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

    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        struct NullableRes;

        impl<'de> Decode<'de> for NullableRes {
            type Output = bytes::Bytes;

            #[inline]
            fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
            where
                B: bytes::Buf,
            {
                let remaining = buf.remaining();
                if remaining <= super::QUERY_MAX_PAYLOAD_LEN {
                    // this was changed in 1.20.2 so the bytes are empty
                    buf.advance(remaining);
                    Ok(Bytes::new())
                } else {
                    Err(anyhow::anyhow!(
                        "payload may not be larger than {} bytes",
                        super::QUERY_MAX_PAYLOAD_LEN
                    ))
                }
            }
        }

        let qid = VarI32::decode(buf)?;
        let res = Option::<NullableRes>::decode(buf)?;

        Ok(Self { query_id: qid, res })
    }
}

impl<L> super::Packet<L> for LoginQueryRes where L: listener::Accept<Self> {}

pub struct LoginKey {
    encrypted_secret_key: Bytes,

    /// The nonce value.
    ///
    /// This value is either encrypted (the left side of {@code Either}) or signed
    /// (the right side). If encrypted, then it must be done so using the server's public key
    /// and the server verifies it by decrypting and comparing nonces. If signed, then it must
    /// be done so using the user's private key provided from Mojang's server, and the server
    /// verifies by checking if the reconstructed data can be verified using the public key.
    nonce: Bytes,
}

impl Encode for LoginKey {
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        self.encrypted_secret_key.encode(buf)?;
        self.nonce.encode(buf)
    }
}

impl<'de> Decode<'de> for LoginKey {
    type Output = LoginKey;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
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
    #[inline]
    pub fn new<R>(
        rng: &mut R,
        secret_key: &[u8],
        public_key: &rsa::RsaPublicKey,
        nonce: &[u8],
    ) -> anyhow::Result<LoginKey>
    where
        R: rsa::rand_core::CryptoRngCore,
    {
        Ok(Self {
            encrypted_secret_key: public_key
                .encrypt(rng, rsa::pkcs1v15::Pkcs1v15Encrypt, secret_key)?
                .try_into()?,
            nonce: public_key
                .encrypt(rng, rsa::pkcs1v15::Pkcs1v15Encrypt, nonce)?
                .try_into()?,
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

pub struct QueryPing {
    start_time: u64,
}

impl QueryPing {
    pub fn start_time(&self) -> u64 {
        self.start_time
    }
}

impl Encode for QueryPing {
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        buf.put_i64(self.start_time as i64);
        Ok(())
    }
}

impl<'de> Decode<'de> for QueryPing {
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

impl<L> super::Packet<L> for QueryPing where L: listener::Accept<Self> {}

pub struct QueryReq;

impl Encode for QueryReq {
    #[inline]
    fn encode<B>(&self, _buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        Ok(())
    }
}

impl<'de> Decode<'de> for QueryReq {
    type Output = Self;

    #[inline]
    fn decode<B>(_buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(Self)
    }
}

impl<L> super::Packet<L> for QueryReq where L: listener::Accept<Self> {}
