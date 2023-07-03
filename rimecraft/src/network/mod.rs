use crate::prelude::*;

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
            fastnbt_rc::to_writer(&mut vec, self.0)?;
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
            Ok(T::deserialize(&mut fastnbt_rc::de::Deserializer::new(
                crate::nbt::BufInput(buf),
                fastnbt_rc::DeOpts::new(),
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

    impl<K, V> Encode for hashbrown::HashMap<K, V>
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

    impl<'de, K, V, OK, OV> Decode<'de> for hashbrown::HashMap<K, V>
    where
        K: for<'a> Decode<'a, Output = OK>,
        V: for<'a> Decode<'a, Output = OV>,
        OK: Hash + Eq,
    {
        type Output = hashbrown::HashMap<OK, OV>;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            let len = crate::util::VarI32::decode(buf)? as usize;
            let mut map = hashbrown::HashMap::with_capacity(len);

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

    impl Encode for BlockPos {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_i64((*self).into());
            Ok(())
        }
    }

    impl<'de> Decode<'de> for BlockPos {
        type Output = BlockPos;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_i64().into())
        }
    }

    impl Encode for crate::util::math::ChunkPos {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_i64((*self).into());
            Ok(())
        }
    }

    impl<'de> Decode<'de> for crate::util::math::ChunkPos {
        type Output = crate::util::math::ChunkPos;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Ok(buf.get_i64().into())
        }
    }

    impl Encode for uuid::Uuid {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            let (a, b) = self.as_u64_pair();
            buf.put_u64(a);
            buf.put_u64(b);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for uuid::Uuid {
        type Output = uuid::Uuid;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            let a = buf.get_u64();
            let b = buf.get_u64();
            Ok(uuid::Uuid::from_u64_pair(a, b))
        }
    }

    impl Encode for crate::nbt::NbtCompound {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            Nbt(self).encode(buf)
        }
    }

    impl<'de> Decode<'de> for crate::nbt::NbtCompound {
        type Output = crate::nbt::NbtCompound;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Nbt::decode(buf)
        }
    }

    impl Encode for glam::Vec3 {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_f32(self.x);
            buf.put_f32(self.y);
            buf.put_f32(self.z);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for glam::Vec3 {
        type Output = glam::Vec3;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            let x = buf.get_f32();
            let y = buf.get_f32();
            let z = buf.get_f32();
            Ok(glam::Vec3 { x, y, z })
        }
    }

    impl Encode for glam::Quat {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            buf.put_f32(self.x);
            buf.put_f32(self.y);
            buf.put_f32(self.z);
            buf.put_f32(self.w);
            Ok(())
        }
    }

    impl<'de> Decode<'de> for glam::Quat {
        type Output = glam::Quat;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            let x = buf.get_f32();
            let y = buf.get_f32();
            let z = buf.get_f32();
            let w = buf.get_f32();
            Ok(glam::Quat::from_xyzw(x, y, z, w))
        }
    }

    impl Encode for crate::item::ItemStack {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            if self.is_empty() {
                false.encode(buf)
            } else {
                true.encode(buf)?;
                self.item().encode(buf)?;
                buf.put_u8(self.count);

                match self.nbt() {
                    Some(map) => {
                        if self.is_damageable() || self.item().descriptor().sync_nbt {
                            return Nbt(map).encode(buf);
                        }
                    }
                    _ => (),
                }

                buf.put_u8(0);
                Ok(())
            }
        }
    }

    impl<'de> Decode<'de> for crate::item::ItemStack {
        type Output = crate::item::ItemStack;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            if !bool::decode(buf)? {
                Ok(crate::item::ItemStack::default())
            } else {
                let item = crate::item::Item::decode(buf)?;
                let mut stack = crate::item::ItemStack::new(&item, buf.get_u8());
                stack.set_nbt(Some(Nbt::decode(buf)?));
                Ok(stack)
            }
        }
    }

    impl Encode for Identifier {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            self.to_string().encode(buf)
        }
    }

    impl<'de> Decode<'de> for Identifier {
        type Output = Identifier;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            Identifier::try_parse(&String::decode(buf)?)
        }
    }

    impl<T> Encode for crate::registry::RegistryKey<T> {
        fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
        where
            B: bytes::BufMut,
        {
            self.value().encode(buf)
        }
    }

    impl<'de, T> Decode<'de> for crate::registry::RegistryKey<T>
    where
        T: RegistryAccess + 'static,
    {
        type Output = crate::registry::RegistryKey<T>;

        fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
        where
            B: bytes::Buf,
        {
            let id = Identifier::decode(buf)?;
            Ok(crate::registry::RegistryKey::new(&T::registry().key, id))
        }
    }
}
