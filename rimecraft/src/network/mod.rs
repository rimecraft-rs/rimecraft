use crate::registry::Registration;

/// Describes types that can be encoded into a packet buffer.
pub trait Encode {
    /// Encode into a buffer.
    fn encode<B: bytes::BufMut>(&self, buf: &mut B) -> anyhow::Result<()>;
}

/// Describes types that can be decoded from a packet buffer.
/// The `'de` lifetime can be used sometimes, like with serde.
pub trait Decode<'de> {
    /// The resulting type.
    type Output;

    /// Decode from a buffer.
    fn decode<B: bytes::Buf>(buf: &'de mut B) -> anyhow::Result<Self::Output>;
}

/// Layer for encoding and decoding in nbt binary format for packets.
pub struct Nbt<'a, T>(pub &'a T);

impl<'a, T: serde::Serialize> Encode for Nbt<'a, T> {
    fn encode<B: bytes::BufMut>(&self, buf: &mut B) -> anyhow::Result<()> {
        let mut vec = Vec::new();
        fastnbt::to_writer(&mut vec, self.0)?;
        buf.put_slice(&vec);
        Ok(())
    }
}

impl<'de, T: serde::Deserialize<'de>> Decode<'de> for Nbt<'_, T> {
    type Output = T;

    fn decode<B: bytes::Buf>(buf: &'de mut B) -> anyhow::Result<Self::Output> {
        Ok(T::deserialize(&mut fastnbt::de::Deserializer::new(
            crate::nbt::BufInput(buf),
            fastnbt::DeOpts::new(),
        ))?)
    }
}

/// Layer for encoding and decoding in json utf8 for packets.
pub struct Json<'a, T>(pub &'a T);

impl<'a, T: serde::Serialize> Encode for Json<'a, T> {
    fn encode<B: bytes::BufMut>(&self, buf: &mut B) -> anyhow::Result<()> {
        serde_json::to_string(&self.0)?.encode(buf)
    }
}

impl<'a, 'de, T: serde::de::DeserializeOwned> Decode<'de> for Json<'a, T> {
    type Output = T;

    fn decode<B: bytes::Buf>(buf: &'de mut B) -> anyhow::Result<Self::Output> {
        let len = crate::util::VarI32::decode(buf)? as usize;
        let mut vec = Vec::new();

        for _ in 0..len {
            vec.push(buf.get_u8());
        }
        Ok(serde_json::from_reader(vec.as_slice())?)
    }
}

impl Encode for crate::util::VarI32 {
    fn encode<B: bytes::BufMut>(&self, buf: &mut B) -> anyhow::Result<()> {
        let mut value = self.0 as u32;

        loop {
            let part = value as u8;
            value >>= 7;

            if value == 0 {
                buf.put_u8(part & 0x7f);
                break Ok(());
            } else {
                buf.put_u8(part | 0x80);
            }
        }
    }
}

impl<'de> Decode<'de> for crate::util::VarI32 {
    type Output = i32;

    fn decode<B: bytes::Buf>(buf: &'de mut B) -> anyhow::Result<Self::Output> {
        let mut value = 0b0;
        let mut pos = 0b0;

        loop {
            let byte = buf.get_u8();
            value |= ((byte & 0x7f) as i32) << pos;

            if (byte & 0x80) == 0 {
                return Ok(value);
            }

            pos += 7;

            if pos >= 32 {
                return Err(anyhow::anyhow!("VarI32 too big"));
            }
        }
    }
}

impl Encode for str {
    fn encode<B: bytes::BufMut>(&self, buf: &mut B) -> anyhow::Result<()> {
        let bs = self.as_bytes();
        crate::util::VarI32(bs.len() as i32).encode(buf)?;
        buf.put_slice(bs);
        Ok(())
    }
}

impl Encode for String {
    fn encode<B: bytes::BufMut>(&self, buf: &mut B) -> anyhow::Result<()> {
        str::encode(&self, buf)
    }
}

impl<'de> Decode<'de> for String {
    type Output = String;

    fn decode<B: bytes::Buf>(buf: &'de mut B) -> anyhow::Result<Self::Output> {
        let len = crate::util::VarI32::decode(buf)? as usize;
        let mut vec = Vec::new();

        for _ in 0..len {
            vec.push(buf.get_u8());
        }

        Ok(String::from_utf8(vec)?)
    }
}

impl<T: Registration> Encode for T {
    fn encode<B: bytes::BufMut>(&self, buf: &mut B) -> anyhow::Result<()> {
        crate::util::VarI32(self.raw_id() as i32).encode(buf)
    }
}
