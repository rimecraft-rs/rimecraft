use std::{convert::Infallible, hash::Hash, string::FromUtf8Error};

use crate::{error::*, *};

impl Encode for bytes::Bytes {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        VarI32(self.len() as i32).encode(&mut buf)?;
        buf.put_slice(&self[..]);
        Ok(())
    }
}

impl Decode for bytes::Bytes {
    type Output = bytes::Bytes;

    type Error = VarI32TooBigError;

    #[inline]
    fn decode<B>(mut buf: B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(&mut buf)? as usize;
        Ok(buf.copy_to_bytes(len))
    }
}

macro_rules! edcode_primitive {
    ($($t:ty => $fe:ident, $fd:ident),*) => {
        $(
            impl Encode for $t {
                type Error = Infallible;

                #[inline]
                fn encode<B: bytes::BufMut>(&self, mut buf: B) -> Result<(), Self::Error> {
                    buf.$fe(*self);
                    Ok(())
                }
            }

            impl Decode for $t {
                type Output = $t;
                type Error = Infallible;

                #[inline]
                fn decode<B: bytes::Buf>(mut buf: B) -> Result<Self::Output, Self::Error>{
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
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        buf.put_u8(if *self { 1 } else { 0 });
        Ok(())
    }
}

impl Decode for bool {
    type Output = bool;

    type Error = Infallible;

    #[inline]
    fn decode<B>(mut buf: B) -> Result<Self::Output, Self::Error>
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
    type Error = fastnbt::error::Error;

    #[inline]
    fn encode<B>(&self, buf: B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        fastnbt::to_writer(bytes::BufMut::writer(buf), &self.0)
    }
}

#[cfg(feature = "nbt")]
impl<T> Decode for Nbt<T>
where
    T: for<'a> serde::Deserialize<'a>,
{
    type Output = T;

    type Error = fastnbt::error::Error;

    #[inline]
    fn decode<B>(buf: B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        fastnbt::from_reader(bytes::Buf::reader(buf))
    }
}

#[cfg(feature = "json")]
impl<T> Encode for Json<T>
where
    T: serde::Serialize,
{
    type Error = serde_json::Error;

    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
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
    type Output = T;

    type Error = ErrorWithVarI32Err<serde_json::Error>;

    fn decode<B>(mut buf: B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(&mut buf)? as usize;
        use std::io::Read;
        serde_json::from_reader(bytes::Buf::reader(buf).take(len as u64))
            .map_err(ErrorWithVarI32Err::Target)
    }
}

impl Encode for VarI32 {
    type Error = Infallible;

    fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
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

impl Decode for VarI32 {
    type Output = i32;

    type Error = VarI32TooBigError;

    fn decode<B>(mut buf: B) -> Result<Self::Output, Self::Error>
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
                return Err(VarI32TooBigError);
            }
        }
    }
}

impl Encode for str {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
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
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        str::encode(self, buf)
    }
}

impl Decode for String {
    type Output = String;

    type Error = ErrorWithVarI32Err<FromUtf8Error>;

    fn decode<B>(mut buf: B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(&mut buf)? as usize;
        let mut vec = vec![0; len];
        buf.copy_to_slice(&mut vec[..]);
        String::from_utf8(vec).map_err(ErrorWithVarI32Err::Target)
    }
}

impl<T> Encode for [T]
where
    T: Encode,
{
    type Error = <T as Encode>::Error;

    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
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

impl<T, O, Err> Decode for Vec<T>
where
    T: for<'a> Decode<Output = O, Error = Err>,
{
    type Output = Vec<O>;

    type Error = ErrorWithVarI32Err<Err>;

    fn decode<B>(mut buf: B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(&mut buf)? as usize;
        let mut vec = Vec::with_capacity(len);

        for _ in 0..len {
            vec.push(T::decode(&mut buf).map_err(ErrorWithVarI32Err::Target)?);
        }

        Ok(vec)
    }
}

impl<K, V> Encode for std::collections::HashMap<K, V>
where
    K: Encode,
    V: Encode,
{
    type Error = EitherError<<K as Encode>::Error, <V as Encode>::Error>;

    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        VarI32(self.len() as i32).encode(&mut buf).unwrap();
        for (key, value) in self.iter() {
            key.encode(&mut buf).map_err(EitherError::A)?;
            value.encode(&mut buf).map_err(EitherError::B)?;
        }
        Ok(())
    }
}

impl<K, V, OK, OV, ErrK, ErrV> Decode for std::collections::HashMap<K, V>
where
    K: for<'a> Decode<Output = OK, Error = ErrK>,
    V: for<'a> Decode<Output = OV, Error = ErrV>,
    OK: Hash + Eq,
{
    type Output = std::collections::HashMap<OK, OV>;

    type Error = ErrorWithVarI32Err<EitherError<ErrK, ErrV>>;

    fn decode<B>(mut buf: B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(&mut buf)? as usize;
        let mut map = std::collections::HashMap::with_capacity(len);
        for _ in 0..len {
            let obj =
                K::decode(&mut buf).map_err(|e| ErrorWithVarI32Err::Target(EitherError::A(e)))?;
            let obj1 =
                V::decode(&mut buf).map_err(|e| ErrorWithVarI32Err::Target(EitherError::B(e)))?;
            map.insert(obj, obj1);
        }
        Ok(map)
    }
}

impl<T> Encode for Option<T>
where
    T: Encode,
{
    type Error = <T as Encode>::Error;

    fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
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

impl<T, OT> Decode for Option<T>
where
    T: Decode<Output = OT>,
{
    type Output = Option<OT>;

    type Error = <T as Decode>::Error;

    fn decode<B>(mut buf: B) -> Result<Self::Output, Self::Error>
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
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
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
    type Output = uuid::Uuid;

    type Error = Infallible;

    #[inline]
    fn decode<B>(mut buf: B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let a = buf.get_u64();
        let b = buf.get_u64();
        Ok(uuid::Uuid::from_u64_pair(a, b))
    }
}

#[cfg(feature = "nbt")]
impl Encode for std::collections::HashMap<String, fastnbt::Value> {
    type Error = fastnbt::error::Error;

    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        Nbt(self).encode(buf)
    }
}

#[cfg(feature = "nbt")]
impl Decode for std::collections::HashMap<String, fastnbt::Value> {
    type Output = std::collections::HashMap<String, fastnbt::Value>;

    type Error = fastnbt::error::Error;

    #[inline]
    fn decode<B>(mut buf: B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        Nbt::decode(buf)
    }
}

#[cfg(feature = "glam")]
impl Encode for glam::Vec3 {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
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
    type Output = glam::Vec3;

    type Error = Infallible;

    #[inline]
    fn decode<B>(mut buf: B) -> Result<Self::Output, Self::Error>
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
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, mut buf: B) -> Result<(), Self::Error>
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
    type Output = glam::Quat;

    type Error = Infallible;

    #[inline]
    fn decode<B>(mut buf: B) -> Result<Self::Output, Self::Error>
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
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, _buf: B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        Ok(())
    }
}

impl Decode for () {
    type Output = ();

    type Error = Infallible;

    #[inline]
    fn decode<B>(_buf: B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        Ok(())
    }
}
