use std::hash::Hash;

use super::*;

impl Encode for bytes::Bytes {
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(buf)? as usize;
        Ok(buf.copy_to_bytes(len))
    }
}

impl Encode for u8 {
    #[inline]
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_u8())
    }
}

impl Encode for i8 {
    #[inline]
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_i8())
    }
}

impl Encode for u16 {
    #[inline]
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_u16())
    }
}

impl Encode for i16 {
    #[inline]
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_i16())
    }
}

impl Encode for u32 {
    #[inline]
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_u32())
    }
}

impl Encode for i32 {
    #[inline]
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_i32())
    }
}

impl Encode for u64 {
    #[inline]
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_u64())
    }
}

impl Encode for i64 {
    #[inline]
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_i64())
    }
}

impl Encode for u128 {
    #[inline]
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_u128())
    }
}

impl Encode for i128 {
    #[inline]
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_i128())
    }
}

impl Encode for f32 {
    #[inline]
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_f32())
    }
}

impl Encode for f64 {
    #[inline]
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(buf.get_f64())
    }
}

impl Encode for bool {
    #[inline]
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
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
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
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

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
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
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
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

    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
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

impl<'de> Decode<'de> for VarI32 {
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
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
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
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        str::encode(self, buf)
    }
}

impl<'de> Decode<'de> for String {
    type Output = String;

    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        let len = VarI32::decode(buf)? as usize;
        let mut vec = Vec::with_capacity(len);

        for _ in 0..len {
            vec.push(buf.get_u8());
        }

        Ok(String::from_utf8(vec)?)
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
        VarI32(self.len() as i32).encode(buf)?;

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
        let len = VarI32::decode(buf)? as usize;
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
        VarI32(self.len() as i32).encode(buf)?;

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
        let len = VarI32::decode(buf)? as usize;
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
        if let Some(value) = self {
            true.encode(buf)?;
            value.encode(buf)
        } else {
            false.encode(buf)
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

#[cfg(feature = "uuid")]
impl Encode for uuid::Uuid {
    #[inline]
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

#[cfg(feature = "uuid")]
impl<'de> Decode<'de> for uuid::Uuid {
    type Output = uuid::Uuid;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
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
    #[inline]
    fn encode<B>(&self, buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        Nbt(self).encode(buf)
    }
}

#[cfg(feature = "nbt")]
impl<'de> Decode<'de> for std::collections::HashMap<String, fastnbt::Value> {
    type Output = std::collections::HashMap<String, fastnbt::Value>;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Nbt::decode(buf)
    }
}

#[cfg(feature = "glam")]
impl Encode for glam::Vec3 {
    #[inline]
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

#[cfg(feature = "glam")]
impl<'de> Decode<'de> for glam::Vec3 {
    type Output = glam::Vec3;

    #[inline]
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

#[cfg(feature = "glam")]
impl Encode for glam::Quat {
    #[inline]
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

#[cfg(feature = "glam")]
impl<'de> Decode<'de> for glam::Quat {
    type Output = glam::Quat;

    #[inline]
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

impl super::Encode for () {
    #[inline]
    fn encode<B>(&self, _buf: &mut B) -> anyhow::Result<()>
    where
        B: bytes::BufMut,
    {
        Ok(())
    }
}

impl<'de> super::Decode<'de> for () {
    type Output = ();

    #[inline]
    fn decode<B>(_buf: &'de mut B) -> anyhow::Result<Self::Output>
    where
        B: bytes::Buf,
    {
        Ok(())
    }
}
