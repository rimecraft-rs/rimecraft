use anyhow::Ok;

use rsa::RsaPrivateKey;

use crate::net::{listener, Decode, Encode};

pub struct Handshake {
    proto_ver: i32,
    addr: String,
    port: u16,
    intended_state: crate::net::State,
}

impl Handshake {
    pub fn new(addr: String, port: u16, intended_state: crate::net::State) -> Self {
        ///TODO: Need to implement net.minecraft.SharedConstants
        const PROTO_VER: i32 = 114514;

        Self {
            proto_ver: PROTO_VER,
            addr,
            port,
            intended_state,
        }
    }
}

impl Encode for Handshake {
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        crate::VarInt(self.proto_ver).encode(buf)?;
        self.addr.encode(buf)?;
        self.port.encode(buf)?;
        crate::VarInt(self.intended_state as i32).encode(buf)?;
        Ok(())
    }
}

impl<'de> Decode<'de> for Handshake {
    type Output = Self;

    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        let proto_ver = crate::VarInt::decode(buf)?;
        let addr = String::decode(buf)?;
        let port = i16::decode(buf)? as u16;
        let state = crate::VarInt::decode(buf)?;

        Ok(Self {
            proto_ver,
            addr,
            port,
            intended_state: crate::net::State::from_id(state).unwrap(),
        })
    }
}

impl<L> super::Packet<L> for Handshake where L: listener::Accept<Self> {}

pub struct LoginHello {
    name: String,
    uuid: Option<uuid::Uuid>,
}

impl Encode for LoginHello {
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        self.name.encode(buf)?;
        self.uuid.encode(buf)
    }
}

impl<'de> Decode<'de> for LoginHello {
    type Output = Self;

    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        let name = String::decode(buf)?;
        let uuid = Option::<uuid::Uuid>::decode(buf)?;

        Ok(Self { name, uuid })
    }
}

impl<L> super::Packet<L> for LoginHello where L: listener::Accept<Self> {}

pub struct LoginQueryRes {
    query_id: i32,
    res: Option<bytes::Bytes>,
}

impl LoginQueryRes {
    pub fn query_id(&self) -> i32 {
        self.query_id
    }

    pub fn response(&self) -> Option<&bytes::Bytes> {
        self.res.as_ref()
    }
}

impl Encode for LoginQueryRes {
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        crate::VarInt(self.query_id).encode(buf)?;

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

            fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
            where
                B: bytes::Buf,
            {
                const MAX_PAYLOAD_LEN: usize = 1048576;

                if (0..=MAX_PAYLOAD_LEN).contains(&buf.remaining()) {
                    bytes::Bytes::decode(buf)
                } else {
                    Err(anyhow::anyhow!(
                        "payload may not be larger than {MAX_PAYLOAD_LEN} bytes"
                    ))
                }
            }
        }

        let qid = crate::VarInt::decode(buf)?;
        let res = Option::<NullableRes>::decode(buf)?;

        Ok(Self { query_id: qid, res })
    }
}

impl<L> super::Packet<L> for LoginQueryRes where L: listener::Accept<Self> {}

pub struct LoginKey {
    encrypted_secret_key: Vec<u8>,

    /// The nonce value.
    ///
    /// This value is either encrypted (the left side of {@code Either}) or signed
    /// (the right side). If encrypted, then it must be done so using the server's public key
    /// and the server verifies it by decrypting and comparing nonces. If signed, then it must
    /// be done so using the user's private key provided from Mojang's server, and the server
    /// verifies by checking if the reconstructed data can be verified using the public key.
    nonce: Vec<u8>,
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
        let encrypted_secret_key = Vec::<u8>::decode(buf)?;
        let nonce = Vec::<u8>::decode(buf)?;

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
            encrypted_secret_key: public_key.encrypt(
                rng,
                rsa::pkcs1v15::Pkcs1v15Encrypt,
                secret_key,
            )?,
            nonce: public_key.encrypt(rng, rsa::pkcs1v15::Pkcs1v15Encrypt, nonce)?,
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
            .map_or(false, |value| nonce == &value)
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
            B: bytes::BufMut {
        (self.start_time as i64).encode(buf)
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
            B: bytes::BufMut {
        Ok(())
    }
}

impl<'de> Decode<'de> for QueryReq {
    type Output = Self;

    #[inline]
    fn decode<B>(_buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf {
        Ok(Self)
    }
}

impl<L> super::Packet<L> for QueryReq where L: listener::Accept<Self> {}
