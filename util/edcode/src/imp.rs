use std::{convert::Infallible, hash::Hash, string::FromUtf8Error};

use crate::{error::*, *};

impl Encode for bytes::Bytes {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        VarI32(self.len() as i32).encode(buf)?;
        buf.put_slice(&self[..]);
        Ok(())
    }
}

impl<'de> Decode<'de> for bytes::Bytes {
    type Output = bytes::Bytes;

    type Error = VarI32TooBigError;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(buf)? as usize;
        Ok(buf.copy_to_bytes(len))
    }
}

macro_rules! edcode_primitive {
    ($($t:ty => $fe:ident, $fd:ident),*) => {
        $(
            impl Encode for $t {
                type Error = Infallible;

                #[inline]
                fn encode<B: bytes::BufMut>(&self, buf: &mut B) -> Result<(), Self::Error> {
                    buf.$fe(*self);
                    Ok(())
                }
            }

            impl<'de> Decode<'de> for $t {
                type Output = $t;
                type Error = Infallible;

                #[inline]
                fn decode<B: bytes::Buf>(buf: &'de mut B) -> Result<Self::Output, Self::Error>{
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
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        buf.put_u8(if *self { 1 } else { 0 });
        Ok(())
    }
}

impl<'de> Decode<'de> for bool {
    type Output = bool;

    type Error = Infallible;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_u8() == 0)
    }
}

#[cfg(feature = "nbt")]
impl<T> Encode for Nbt<'_, T>
where
    T: serde::Serialize,
{
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        struct WriteAdapt<'a, T: 'a>(pub &'a mut T);

        impl<T> std::io::Write for WriteAdapt<'_, T>
        where
            T: bytes::BufMut,
        {
            #[inline]
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                unsafe { &mut *(self.0 as *mut T as *mut bytes::buf::Writer<T>) }.write(buf)
            }

            #[inline]
            fn flush(&mut self) -> std::io::Result<()> {
                unsafe { &mut *(self.0 as *mut T as *mut bytes::buf::Writer<T>) }.flush()
            }
        }

        fastnbt::to_writer(WriteAdapt(buf), self.0)?;
        Ok(())
    }
}

#[cfg(feature = "nbt")]
impl<'de, T> Decode<'de> for Nbt<'_, T>
where
    T: serde::Deserialize<'de>,
{
    type Output = T;

    type Error = Infallible;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        struct ReadAdapt<'a, T: 'a>(pub &'a mut T);

        impl<T> std::io::Read for ReadAdapt<'_, T>
        where
            T: bytes::Buf,
        {
            #[inline]
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                unsafe { &mut *(self.0 as *mut T as *mut bytes::buf::Reader<T>) }.read(buf)
            }
        }

        Ok(fastnbt::from_reader(ReadAdapt(buf))?)
    }
}

#[cfg(feature = "json")]
impl<T> Encode for Json<'_, T>
where
    T: serde::Serialize,
{
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        serde_json::to_string(&self.0)?.encode(buf)
    }
}

#[cfg(feature = "json")]
impl<'de, T> Decode<'de> for Json<'_, T>
where
    T: serde::de::DeserializeOwned,
{
    type Output = T;

    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(buf)? as usize;
        let mut vec = Vec::with_capacity(len);

        for _ in 0..len {
            vec.push(buf.get_u8());
        }

        Ok(serde_json::from_reader(vec.as_slice())?)
    }
}

impl Encode for VarI32 {
    type Error = Infallible;

    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
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

impl<'de> Decode<'de> for VarI32 {
    type Output = i32;

    type Error = VarI32TooBigError;

    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
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
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        let bs = self.as_bytes();
        VarI32(bs.len() as i32).encode(buf)?;
        buf.put_slice(bs);
        Ok(())
    }
}

impl Encode for String {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        str::encode(self, buf)
    }
}

impl<'de> Decode<'de> for String {
    type Output = String;

    type Error = ErrorWithVarI32Len<FromUtf8Error>;

    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(buf)? as usize;
        let mut vec = vec![0; len];
        buf.copy_to_slice(&mut vec[..]);
        Ok(String::from_utf8(vec).map_err(ErrorWithVarI32Len::Target)?)
    }
}

impl<T> Encode for [T]
where
    T: Encode,
{
    type Error = <T as Encode>::Error;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        VarI32(self.len() as i32).encode(buf).unwrap();
        for object in self.iter() {
            object.encode(buf)?;
        }
        Ok(())
    }
}

impl<'de, T, O, Err> Decode<'de> for Vec<T>
where
    T: for<'a> Decode<'a, Output = O, Error = Err>,
{
    type Output = Vec<O>;

    type Error = ErrorWithVarI32Len<Err>;

    fn decode<B>(mut buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(&mut buf)? as usize;
        let mut vec = Vec::with_capacity(len);

        for _ in 0..len {
            vec.push(T::decode(&mut buf).map_err(ErrorWithVarI32Len::Target)?);
        }

        Ok(vec)
    }
}

impl<K, V> Encode for std::collections::HashMap<K, V>
where
    K: Encode,
    V: Encode,
{
    type Error = KvError<<K as Encode>::Error, <V as Encode>::Error>;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        VarI32(self.len() as i32).encode(buf).unwrap();
        for (key, value) in self.iter() {
            key.encode(buf).map_err(KvError::Key)?;
            value.encode(buf).map_err(KvError::Value)?;
        }
        Ok(())
    }
}

impl<'de, K, V, OK, OV, ErrK, ErrV> Decode<'de> for std::collections::HashMap<K, V>
where
    K: for<'a> Decode<'a, Output = OK, Error = ErrK>,
    V: for<'a> Decode<'a, Output = OV, Error = ErrV>,
    OK: Hash + Eq,
{
    type Output = std::collections::HashMap<OK, OV>;

    type Error = ErrorWithVarI32Len<KvError<ErrK, ErrV>>;

    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(buf)? as usize;
        let mut map = std::collections::HashMap::with_capacity(len);
        for _ in 0..len {
            let obj = K::decode(buf).map_err(|e| ErrorWithVarI32Len::Target(KvError::Key(e)))?;
            let obj1 = V::decode(buf).map_err(|e| ErrorWithVarI32Len::Target(KvError::Value(e)))?;
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

    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        if let Some(value) = self {
            true.encode(buf).unwrap();
            value.encode(buf)
        } else {
            false.encode(buf).unwrap();
            Ok(())
        }
    }
}

impl<'de, T, OT> Decode<'de> for Option<T>
where
    T: Decode<'de, Output = OT>,
{
    type Output = Option<OT>;

    type Error = <T as Decode<'de>>::Error;

    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        Ok(if bool::decode(buf).unwrap() {
            Some(T::decode(buf)?)
        } else {
            None
        })
    }
}

#[cfg(feature = "uuid")]
impl Encode for uuid::Uuid {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
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
impl<'de> Decode<'de> for uuid::Uuid {
    type Output = uuid::Uuid;

    type Error = Infallible;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
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
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        Nbt(self).encode(buf)
    }
}

#[cfg(feature = "nbt")]
impl<'de> Decode<'de> for std::collections::HashMap<String, fastnbt::Value> {
    type Output = std::collections::HashMap<String, fastnbt::Value>;

    type Error = Infallible;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
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
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
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
impl<'de> Decode<'de> for glam::Vec3 {
    type Output = glam::Vec3;

    type Error = Infallible;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
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
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
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
impl<'de> Decode<'de> for glam::Quat {
    type Output = glam::Quat;

    type Error = Infallible;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
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

impl super::Encode for () {
    type Error = Infallible;

    #[inline]
    fn encode<B>(&self, _buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        Ok(())
    }
}

impl<'de> super::Decode<'de> for () {
    type Output = ();

    type Error = Infallible;

    #[inline]
    fn decode<B>(_buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        Ok(())
    }
}
