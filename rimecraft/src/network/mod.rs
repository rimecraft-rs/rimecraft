/// Describes types that can be encoded into a packet buffer.
pub trait Encode {
    /// Encode into a buffer.
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut;
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

/// Layer for encoding and decoding in nbt binary format for packets.
pub struct Nbt<'a, T>(pub &'a T);

/// Layer for encoding and decoding in json utf8 for packets.
pub struct Json<'a, T>(pub &'a T);

mod packet_buf_impl {
    use std::{hash::Hash, ops::Deref};

    use crate::registry::{Registration, RegistryAccess};

    use super::*;

    impl Encode for u8 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_u8(*self);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for u8 {
        type Output = u8;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_u8())
        }
    }

    impl Encode for i8 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_i8(*self);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for i8 {
        type Output = i8;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_i8())
        }
    }

    impl Encode for u16 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_u16(*self);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for u16 {
        type Output = u16;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_u16())
        }
    }

    impl Encode for i16 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_i16(*self);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for i16 {
        type Output = i16;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_i16())
        }
    }

    impl Encode for u32 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_u32(*self);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for u32 {
        type Output = u32;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_u32())
        }
    }

    impl Encode for i32 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_i32(*self);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for i32 {
        type Output = i32;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_i32())
        }
    }

    impl Encode for u64 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_u64(*self);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for u64 {
        type Output = u64;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_u64())
        }
    }

    impl Encode for i64 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_i64(*self);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for i64 {
        type Output = i64;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_i64())
        }
    }

    impl Encode for u128 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_u128(*self);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for u128 {
        type Output = u128;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_u128())
        }
    }

    impl Encode for i128 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_i128(*self);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for i128 {
        type Output = i128;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_i128())
        }
    }

    impl Encode for f32 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_f32(*self);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for f32 {
        type Output = f32;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_f32())
        }
    }

    impl Encode for f64 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_f64(*self);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for f64 {
        type Output = f64;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_f64())
        }
    }

    impl Encode for bool {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_u8(if *self { 1 } else { 0 });
            Ok(())
        }
    }

    impl<'de> Decode<'de> for bool {
        type Output = bool;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_u8() == 0)
        }
    }

    impl<T> Encode for Nbt<'_, T>
    where
        T: serde::Serialize,
    {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            let mut vec = Vec::new();
            fastnbt::to_writer(&mut vec, self.0)?;
            buf.put_slice(&vec);
            Ok(())
        }
    }

    impl<'de, T> Decode<'de> for Nbt<'_, T>
    where
        T: serde::Deserialize<'de>,
    {
        type Output = T;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(T::deserialize(&mut fastnbt::de::Deserializer::new(
                crate::nbt::BufInput(buf),
                fastnbt::DeOpts::new(),
            ))?)
        }
    }

    impl<T> Encode for Json<'_, T>
    where
        T: serde::Serialize,
    {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            serde_json::to_string(&self.0)?.encode(buf)
        }
    }

    impl<'de, T> Decode<'de> for Json<'_, T>
    where
        T: serde::de::DeserializeOwned,
    {
        type Output = T;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            let len = crate::util::VarI32::decode(buf)? as usize;
            let mut vec = Vec::new();

            for _ in 0..len {
                vec.push(buf.get_u8());
            }
            Ok(serde_json::from_reader(vec.as_slice())?)
        }
    }

    impl Encode for crate::util::VarI32 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
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

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
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
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            let bs = self.as_bytes();
            crate::util::VarI32(bs.len() as i32).encode(buf)?;
            buf.put_slice(bs);
            Ok(())
        }
    }

    impl Encode for String {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            str::encode(&self, buf)
        }
    }

    impl<'de> Decode<'de> for String {
        type Output = String;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            let len = crate::util::VarI32::decode(buf)? as usize;
            let mut vec = Vec::new();

            for _ in 0..len {
                vec.push(buf.get_u8());
            }

            Ok(String::from_utf8(vec)?)
        }
    }

    impl<T> Encode for T
    where
        T: Registration,
    {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            crate::util::VarI32(self.raw_id() as i32).encode(buf)
        }
    }

    impl<'de, T> Decode<'de> for T
    where
        T: RegistryAccess + Clone + 'static,
    {
        type Output = T;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            let id = crate::util::VarI32::decode(buf)? as usize;
            match T::registry().get_from_raw(id) {
                Some(value) => Ok(value.deref().clone()),
                None => {
                    if T::registry().is_defaulted() {
                        Ok(T::registry().get_from_raw(id).unwrap().deref().clone())
                    } else {
                        Err(anyhow::anyhow!("Raw id {id} not found in registry"))
                    }
                }
            }
        }
    }

    impl<T> Encode for [T]
    where
        T: Encode,
    {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            crate::util::VarI32(self.len() as i32).encode(buf)?;

            for object in self.iter() {
                object.encode(buf)?;
            }

            Ok(())
        }
    }

    impl<'de, T, O> Decode<'de> for Vec<T>
    where
        T: for<'a> Decode<'a, Output = O>,
    {
        type Output = Vec<O>;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            let len = crate::util::VarI32::decode(buf)? as usize;
            let mut vec = Vec::with_capacity(len);

            for _ in 0..len {
                vec.push(T::decode(buf)?);
            }

            Ok(vec)
        }
    }

    impl<K, V> Encode for std::collections::HashMap<K, V>
    where
        K: Encode,
        V: Encode,
    {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            crate::util::VarI32(self.len() as i32).encode(buf)?;

            for (key, value) in self.iter() {
                key.encode(buf)?;
                value.encode(buf)?;
            }

            Ok(())
        }
    }

    impl<'de, K, V, OK, OV> Decode<'de> for std::collections::HashMap<K, V>
    where
        K: for<'a> Decode<'a, Output = OK>,
        V: for<'a> Decode<'a, Output = OV>,
        OK: Hash + Eq,
    {
        type Output = std::collections::HashMap<OK, OV>;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            let len = crate::util::VarI32::decode(buf)? as usize;
            let mut map = std::collections::HashMap::with_capacity(len);

            for _ in 0..len {
                let obj = K::decode(buf)?;
                let obj1 = V::decode(buf)?;
                map.insert(obj, obj1);
            }

            Ok(map)
        }
    }

    impl<T> Encode for Option<T>
    where
        T: Encode,
    {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            match self {
                Some(value) => {
                    true.encode(buf)?;
                    value.encode(buf)
                }
                None => false.encode(buf),
            }
        }
    }

    impl<'de, T, OT> Decode<'de> for Option<T>
    where
        T: Decode<'de, Output = OT>,
    {
        type Output = Option<OT>;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(if bool::decode(buf)? {
                Some(T::decode(buf)?)
            } else {
                None
            })
        }
    }
}
