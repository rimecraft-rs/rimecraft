use std::{collections::HashMap, hash::Hash};

use crate::*;

impl Encode for bytes::Bytes {
    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        VarI32(self.len() as i32).encode(&mut buf)?;
        buf.put_slice(&self[..]);
        Ok(())
    }
}

impl Decode for bytes::Bytes {
    #[inline]
    fn decode<B>(mut buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(&mut buf)?.0 as usize;
        Ok(buf.copy_to_bytes(len))
    }
}

macro_rules! edcode_primitive {
    ($($t:ty => $fe:ident, $fd:ident),*) => {
        $(
            impl Encode for $t {
                #[inline(always)]
                fn encode<B: bytes::BufMut>(&self, mut buf: B) -> Result<(), io::Error> {
                    buf.$fe(*self);
                    Ok(())
                }
            }

            impl Decode for $t {
                #[inline(always)]
                fn decode<B: bytes::Buf>(mut buf: B) -> Result<Self, io::Error>{
                    Ok(buf.$fd())
                }
            }
        )*
    };
}

edcode_primitive! {
    u8 => put_u8, get_u8,
    u16 => put_u16, get_u16,
    u32 => put_u32, get_u32,
    u64 => put_u64, get_u64,
    i8 => put_i8, get_i8,
    i16 => put_i16, get_i16,
    i32 => put_i32, get_i32,
    i64 => put_i64, get_i64,
    f32 => put_f32, get_f32,
    f64 => put_f64, get_f64
}

impl Encode for bool {
    #[inline(always)]
    fn encode<B>(&self, mut buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        buf.put_u8(if *self { 1 } else { 0 });
        Ok(())
    }
}

impl Decode for bool {
    #[inline(always)]
    fn decode<B>(mut buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_u8() == 0)
    }
}

#[cfg(feature = "nbt")]
impl<T> Encode for Nbt<T>
where
    T: serde::Serialize,
{
    #[inline]
    fn encode<B>(&self, buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        fastnbt::to_writer(bytes::BufMut::writer(buf), &self.0)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}

#[cfg(feature = "nbt")]
impl<T> Decode for Nbt<T>
where
    T: for<'a> serde::Deserialize<'a>,
{
    #[inline]
    fn decode<B>(buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        fastnbt::from_reader(bytes::Buf::reader(buf))
            .map(Self)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}

#[cfg(feature = "json")]
impl<T> Encode for Json<T>
where
    T: serde::Serialize,
{
    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        let vec = serde_json::to_vec(&self.0)?;
        VarI32(vec.len() as i32).encode(&mut buf).unwrap();
        buf.put_slice(&vec);
        Ok(())
    }
}

#[cfg(feature = "json")]
impl<T> Decode for Json<T>
where
    T: for<'a> serde::de::Deserialize<'a>,
{
    fn decode<B>(mut buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(&mut buf)?.0 as usize;
        use std::io::Read;
        serde_json::from_reader(bytes::Buf::reader(buf).take(len as u64))
            .map(Self)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}

impl Encode for VarI32 {
    fn encode<B>(&self, mut buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        let mut i = self.0;
        while i & -128 != 0 {
            buf.put_u8((i & 127 | 128) as u8);
            i >>= 7;
        }
        buf.put_u8(i as u8);
        Ok(())
    }
}

impl Decode for VarI32 {
    fn decode<B>(mut buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        let mut i = 0;
        let mut j = 0;
        loop {
            let b = buf.get_u8();
            i |= ((b & 127) as i32) << (j * 7);
            j += 1;
            if j > 5 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarI32 too long",
                ));
            }
            if (b & 128) != 128 {
                return Ok(Self(i));
            }
        }
    }
}

impl Encode for str {
    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        let bs = self.as_bytes();
        VarI32(bs.len() as i32).encode(&mut buf)?;
        buf.put_slice(bs);
        Ok(())
    }
}

impl Encode for String {
    #[inline]
    fn encode<B>(&self, buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        str::encode(self, buf)
    }
}

impl Decode for String {
    fn decode<B>(mut buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(&mut buf)?.0 as usize;
        let mut vec = vec![0; len];
        buf.copy_to_slice(&mut vec[..]);
        String::from_utf8(vec).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

impl<T> Encode for [T]
where
    T: Encode,
{
    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        VarI32(self.len() as i32).encode(&mut buf).unwrap();
        for object in self.iter() {
            object.encode(&mut buf)?;
        }
        Ok(())
    }
}

impl<T> Decode for Vec<T>
where
    T: Decode,
{
    fn decode<B>(mut buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(&mut buf)?.0 as usize;
        let mut vec = Vec::with_capacity(len);

        for _ in 0..len {
            vec.push(T::decode(&mut buf)?);
        }

        Ok(vec)
    }
}

impl<K, V> Encode for HashMap<K, V>
where
    K: Encode,
    V: Encode,
{
    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        VarI32(self.len() as i32).encode(&mut buf).unwrap();
        for (key, value) in self.iter() {
            key.encode(&mut buf)?;
            value.encode(&mut buf)?;
        }
        Ok(())
    }
}

impl<K, V> Decode for HashMap<K, V>
where
    K: Decode + Hash + Eq,
    V: Decode,
{
    fn decode<B>(mut buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(&mut buf)?.0 as usize;
        let mut map = HashMap::with_capacity(len);
        for _ in 0..len {
            let obj = K::decode(&mut buf)?;
            let obj1 = V::decode(&mut buf)?;
            map.insert(obj, obj1);
        }
        Ok(map)
    }
}

impl<T> Encode for Option<T>
where
    T: Encode,
{
    fn encode<B>(&self, mut buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        if let Some(value) = self {
            true.encode(&mut buf).unwrap();
            value.encode(&mut buf)
        } else {
            false.encode(&mut buf).unwrap();
            Ok(())
        }
    }
}

impl<T> Decode for Option<T>
where
    T: Decode,
{
    #[allow(clippy::if_then_some_else_none)]
    fn decode<B>(mut buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        Ok(if bool::decode(&mut buf).unwrap() {
            Some(T::decode(&mut buf)?)
        } else {
            None
        })
    }
}

#[cfg(feature = "uuid")]
impl Encode for uuid::Uuid {
    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        let (a, b) = self.as_u64_pair();
        buf.put_u64(a);
        buf.put_u64(b);
        Ok(())
    }
}

#[cfg(feature = "uuid")]
impl Decode for uuid::Uuid {
    #[inline]
    fn decode<B>(mut buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        let a = buf.get_u64();
        let b = buf.get_u64();
        Ok(uuid::Uuid::from_u64_pair(a, b))
    }
}

#[cfg(feature = "nbt")]
impl Encode for HashMap<String, fastnbt::Value> {
    #[inline]
    fn encode<B>(&self, buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        Nbt(self).encode(buf)
    }
}

#[cfg(feature = "nbt")]
impl Decode for HashMap<String, fastnbt::Value> {
    #[inline]
    fn decode<B>(buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        Nbt::decode(buf).map(|nbt| nbt.0)
    }
}

#[cfg(feature = "glam")]
impl Encode for glam::Vec3 {
    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        buf.put_f32(self.x);
        buf.put_f32(self.y);
        buf.put_f32(self.z);
        Ok(())
    }
}

#[cfg(feature = "glam")]
impl Decode for glam::Vec3 {
    #[inline]
    fn decode<B>(mut buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        let x = buf.get_f32();
        let y = buf.get_f32();
        let z = buf.get_f32();
        Ok(glam::Vec3 { x, y, z })
    }
}

#[cfg(feature = "glam")]
impl Encode for glam::Quat {
    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), io::Error>
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

#[cfg(feature = "glam")]
impl Decode for glam::Quat {
    #[inline]
    fn decode<B>(mut buf: B) -> Result<Self, io::Error>
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

impl Encode for () {
    #[inline]
    fn encode<B>(&self, _buf: B) -> Result<(), io::Error>
    where
        B: bytes::BufMut,
    {
        Ok(())
    }
}

impl Decode for () {
    #[inline]
    fn decode<B>(_buf: B) -> Result<Self, io::Error>
    where
        B: bytes::Buf,
    {
        Ok(())
    }
}
