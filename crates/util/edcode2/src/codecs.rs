//! Utilities for encoding and decoding common types.

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    hash::Hash,
};

use bytes::BytesMut;

use crate::*;

/// A variable-length wrapper type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable<T>(pub T);

/// A byte array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ByteArray<T>(pub T);

macro_rules! primitives {
    ($($t:ty => $p:ident, $g:ident),*$(,)?) => {
        $(
        impl<B: BufMut> Encode<B> for $t {
            #[inline]
            fn encode(&self, mut buf: B) -> Result<(), BoxedError<'static>> {
                buf.$p(*self);
                Ok(())
            }
        }

        impl<'de, B: Buf> Decode<'de, B> for $t {
            #[inline]
            fn decode(mut buf: B) -> Result<Self, BoxedError<'de>> {
                Ok(buf.$g())
            }
        }
        )*
    };
}

primitives! {
    u8 => put_u8, get_u8,
    u16 => put_u16, get_u16,
    u32 => put_u32, get_u32,
    u64 => put_u64, get_u64,
    u128 => put_u128, get_u128,

    i8 => put_i8, get_i8,
    i16 => put_i16, get_i16,
    i32 => put_i32, get_i32,
    i64 => put_i64, get_i64,
    i128 => put_i128, get_i128,

    f32 => put_f32, get_f32,
    f64 => put_f64, get_f64,
}

impl<B: BufMut> Encode<B> for bool {
    #[inline]
    fn encode(&self, mut buf: B) -> Result<(), BoxedError<'static>> {
        buf.put_u8(*self as u8);
        Ok(())
    }
}

impl<'de, B: Buf> Decode<'de, B> for bool {
    #[inline]
    fn decode(mut buf: B) -> Result<Self, BoxedError<'de>> {
        Ok(buf.get_u8() != 0)
    }
}

macro_rules! unsigned_variable_primitives {
    ($($t:ty),*$(,)?) => {
        type BitCount = u32;
        const VAR_SHIFT: BitCount = u8::BITS - 1;

        $(
        #[allow(trivial_numeric_casts)]
        impl<B: BufMut> Encode<B> for Variable<$t> {
            fn encode(&self, mut buf: B) -> Result<(), BoxedError<'static>> {
                let Variable(mut i) = *self;
                while i & (<$t>::MAX << VAR_SHIFT) != 0 {
                    buf.put_u8((i & 0b0111_1111 | 0b1000_0000) as u8);
                    i >>= VAR_SHIFT;
                }
                buf.put_u8(i as u8);
                Ok(())
            }
        }

        #[allow(trivial_numeric_casts)]
        impl<'de, B: Buf> Decode<'de, B> for Variable<$t> {
            fn decode(mut buf: B) -> Result<Self, BoxedError<'de>> {
                let mut i: $t = 0;
                let mut shift: BitCount = 0;

                loop {
                    let b = buf.get_u8();
                    i |= ((b & 0b0111_1111) as $t) << shift;
                    shift += VAR_SHIFT;
                    if shift > <$t>::BITS + u8::BITS {
                        return Err("variable integer too large".into());
                    }
                    if b & 0b1000_0000 != 0b1000_0000 {
                        return Ok(Self(i));
                    }
                }
            }
        }
        )*
    };
}

unsigned_variable_primitives! {
    u16, u32, u64, u128,
}

macro_rules! signed_variable_primitives {
    ($($s:ty => $u:ty),*$(,)?) => {
        $(
        impl<B: BufMut> Encode<B> for Variable<$s> {
            #[inline]
            fn encode(&self, buf: B) -> Result<(), BoxedError<'static>> {
                let var = Variable(self.0 as $u);
                var.encode(buf)
            }
        }

        impl<'de, B: Buf> Decode<'de, B> for Variable<$s> {
            #[inline]
            fn decode(buf: B) -> Result<Self, BoxedError<'de>> {
                Ok(Self(Variable::<$u>::decode(buf)?.0 as $s))
            }
        }
        )*
    };
}

signed_variable_primitives! {
    i16 => u16,
    i32 => u32,
    i64 => u64,
    i128 => u128,
}

impl<B: BufMut, T> Encode<B> for [T]
where
    T: for<'a> Encode<&'a mut B>,
{
    fn encode(&self, mut buf: B) -> Result<(), BoxedError<'static>> {
        buf.put_variable(self.len() as u32);
        for item in self {
            item.encode(&mut buf)?;
        }
        Ok(())
    }
}

impl<B: BufMut, T> Encode<B> for Vec<T>
where
    T: for<'a> Encode<&'a mut B>,
{
    #[inline]
    fn encode(&self, buf: B) -> Result<(), BoxedError<'static>> {
        (**self).encode(buf)
    }
}

impl<'de, B: Buf, T> Decode<'de, B> for &mut [T]
where
    T: for<'a> Decode<'de, &'a mut B>,
{
    fn decode_in_place(&mut self, mut buf: B) -> Result<(), BoxedError<'de>> {
        let len = buf.get_variable::<u32>() as usize;
        if self.len() < len {
            return Err(format!(
                "slice too short: received {} items, have {} slots",
                len,
                self.len()
            )
            .into());
        }
        for val in self[..len].iter_mut() {
            val.decode_in_place(&mut buf)?;
        }
        Ok(())
    }

    #[inline]
    fn decode(_buf: B) -> Result<Self, BoxedError<'de>> {
        Err("slices does not support non-in-place decoding".into())
    }

    const SUPPORT_NON_IN_PLACE: bool = false;
}

impl<'de, B: Buf, T> Decode<'de, B> for Vec<T>
where
    T: for<'a> Decode<'de, &'a mut B>,
{
    fn decode(mut buf: B) -> Result<Self, BoxedError<'de>> {
        let len = buf.get_variable::<u32>() as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::decode(&mut buf)?);
        }
        Ok(vec)
    }
}

impl<B: BufMut, T> Encode<B> for Box<[T]>
where
    T: for<'a> Encode<&'a mut B>,
{
    #[inline]
    fn encode(&self, buf: B) -> Result<(), BoxedError<'static>> {
        (**self).encode(buf)
    }
}

impl<'de, B: Buf, T> Decode<'de, B> for Box<[T]>
where
    T: for<'a> Decode<'de, &'a mut B>,
{
    #[inline]
    fn decode(buf: B) -> Result<Self, BoxedError<'de>> {
        Vec::<T>::decode(buf).map(Into::into)
    }
}

impl<B: BufMut, T> Encode<B> for ByteArray<T>
where
    T: AsRef<[u8]>,
{
    #[inline]
    fn encode(&self, mut buf: B) -> Result<(), BoxedError<'static>> {
        let s = self.0.as_ref();
        buf.put_variable(s.len() as u32);
        buf.put_slice(s);
        Ok(())
    }
}

impl<'de, B: Buf> Decode<'de, B> for ByteArray<Vec<u8>> {
    #[inline]
    fn decode(mut buf: B) -> Result<Self, BoxedError<'de>> {
        let len = buf.get_variable::<u32>() as usize;
        let mut vec = vec![0; len];
        buf.copy_to_slice(&mut vec);
        Ok(Self(vec))
    }
}

impl<'de, B: Buf> Decode<'de, B> for ByteArray<Box<[u8]>> {
    #[inline]
    fn decode(buf: B) -> Result<Self, BoxedError<'de>> {
        ByteArray::<Vec<u8>>::decode(buf).map(|ByteArray(vec)| Self(vec.into_boxed_slice()))
    }
}

impl<'de, B: Buf> Decode<'de, &'de mut B> for ByteArray<Cow<'de, [u8]>> {
    fn decode(buf: &'de mut B) -> Result<Self, BoxedError<'de>> {
        let len = buf.get_variable::<u32>() as usize;
        if buf.chunk().len() >= len {
            buf.advance(len);
            Ok(ByteArray(Cow::Borrowed(&B::chunk(buf)[..len])))
        } else {
            ByteArray::<Vec<u8>>::decode(buf).map(|ByteArray(vec)| ByteArray(Cow::Owned(vec)))
        }
    }
}

impl<B: BufMut> Encode<B> for Cow<'_, [u8]> {
    #[inline]
    fn encode(&self, buf: B) -> Result<(), BoxedError<'static>> {
        ByteArray(self.as_ref()).encode(buf)
    }
}

impl<'de, B: Buf> Decode<'de, &'de mut B> for Cow<'de, [u8]> {
    #[inline]
    fn decode(buf: &'de mut B) -> Result<Self, BoxedError<'de>> {
        ByteArray::<Cow<'de, [u8]>>::decode(buf).map(|ByteArray(bytes)| bytes)
    }
}

const MAX_STR_LEN: usize = i16::MAX as usize;

impl<B: BufMut> Encode<B> for str {
    #[inline]
    fn encode(&self, mut buf: B) -> Result<(), BoxedError<'static>> {
        ByteArray(self.as_bytes()).encode(&mut buf)
    }
}

impl<B: BufMut> Encode<B> for String {
    #[inline]
    fn encode(&self, buf: B) -> Result<(), BoxedError<'static>> {
        self.as_str().encode(buf)
    }
}

impl<'de, B: Buf> Decode<'de, B> for String {
    #[inline]
    fn decode(buf: B) -> Result<Self, BoxedError<'de>> {
        let ByteArray(bytes) = ByteArray::<Vec<u8>>::decode(buf)?;
        if bytes.len() > MAX_STR_LEN {
            return Err("string too large".into());
        }
        String::from_utf8(bytes).map_err(Into::into)
    }
}

impl<B: BufMut> Encode<B> for Box<str> {
    #[inline]
    fn encode(&self, buf: B) -> Result<(), BoxedError<'static>> {
        self.as_ref().encode(buf)
    }
}

impl<'de, B: Buf> Decode<'de, B> for Box<str> {
    #[inline]
    fn decode(buf: B) -> Result<Self, BoxedError<'de>> {
        String::decode(buf).map(Into::into)
    }
}

impl<B: BufMut> Encode<B> for Cow<'_, str> {
    #[inline]
    fn encode(&self, buf: B) -> Result<(), BoxedError<'static>> {
        self.as_ref().encode(buf)
    }
}

impl<'de, B: Buf> Decode<'de, &'de mut B> for Cow<'de, str> {
    fn decode(buf: &'de mut B) -> Result<Self, BoxedError<'de>> {
        Cow::<'de, [u8]>::decode(buf)
            .and_then(|cow| {
                if cow.len() > MAX_STR_LEN {
                    Err("string too large".into())
                } else {
                    Ok(cow)
                }
            })
            .and_then(|bytes| match bytes {
                Cow::Borrowed(bytes) => std::str::from_utf8(bytes)
                    .map(Cow::Borrowed)
                    .map_err(Into::into),
                Cow::Owned(bytes) => String::from_utf8(bytes).map(Cow::Owned).map_err(Into::into),
            })
    }
}

impl<B: BufMut, T> Encode<B> for Option<T>
where
    T: for<'a> Encode<&'a mut B>,
{
    #[inline]
    fn encode(&self, mut buf: B) -> Result<(), BoxedError<'static>> {
        match self {
            Some(value) => {
                buf.put_bool(true);
                value.encode(&mut buf)
            }
            None => {
                buf.put_bool(false);
                Ok(())
            }
        }
    }
}

impl<'de, B: Buf, T> Decode<'de, B> for Option<T>
where
    T: for<'a> Decode<'de, &'a mut B>,
{
    #[inline]
    fn decode(mut buf: B) -> Result<Self, BoxedError<'de>> {
        if buf.get_bool() {
            T::decode(&mut buf).map(Some)
        } else {
            Ok(None)
        }
    }
}

impl<B: BufMut, K, V, S> Encode<B> for HashMap<K, V, S>
where
    K: for<'a> Encode<&'a mut B>,
    V: for<'a> Encode<&'a mut B>,
{
    fn encode(&self, mut buf: B) -> Result<(), BoxedError<'static>> {
        buf.put_variable(self.len() as u32);
        for (key, value) in self {
            key.encode(&mut buf)?;
            value.encode(&mut buf)?;
        }
        Ok(())
    }
}

impl<'de, B: Buf, K, V, S> Decode<'de, B> for HashMap<K, V, S>
where
    K: for<'a> Decode<'de, &'a mut B> + Hash + Eq,
    V: for<'a> Decode<'de, &'a mut B>,
    S: Default + std::hash::BuildHasher,
{
    fn decode(mut buf: B) -> Result<Self, BoxedError<'de>> {
        let len = buf.get_variable::<u32>() as usize;
        let mut map = HashMap::with_capacity_and_hasher(len.min(u16::MAX as usize), S::default());
        for _ in 0..len {
            let key = K::decode(&mut buf)?;
            let value = V::decode(&mut buf)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<B: BufMut, K, V> Encode<B> for BTreeMap<K, V>
where
    K: for<'a> Encode<&'a mut B>,
    V: for<'a> Encode<&'a mut B>,
{
    fn encode(&self, mut buf: B) -> Result<(), BoxedError<'static>> {
        buf.put_variable(self.len() as u32);
        for (key, value) in self {
            key.encode(&mut buf)?;
            value.encode(&mut buf)?;
        }
        Ok(())
    }
}

impl<'de, B: Buf, K, V> Decode<'de, B> for BTreeMap<K, V>
where
    K: for<'a> Decode<'de, &'a mut B> + Ord,
    V: for<'a> Decode<'de, &'a mut B>,
{
    fn decode(mut buf: B) -> Result<Self, BoxedError<'de>> {
        let len = buf.get_variable::<u32>() as usize;
        let mut map = BTreeMap::new();
        for _ in 0..len {
            let key = K::decode(&mut buf)?;
            let value = V::decode(&mut buf)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<B: BufMut, T, S> Encode<B> for HashSet<T, S>
where
    T: for<'a> Encode<&'a mut B>,
{
    fn encode(&self, mut buf: B) -> Result<(), BoxedError<'static>> {
        buf.put_variable(self.len() as u32);
        for item in self {
            item.encode(&mut buf)?;
        }
        Ok(())
    }
}

impl<'de, B: Buf, T, S> Decode<'de, B> for HashSet<T, S>
where
    T: for<'a> Decode<'de, &'a mut B> + Hash + Eq,
    S: Default + std::hash::BuildHasher,
{
    fn decode(mut buf: B) -> Result<Self, BoxedError<'de>> {
        let len = buf.get_variable::<u32>() as usize;
        let mut set = HashSet::with_capacity_and_hasher(len.min(u16::MAX as usize), S::default());
        for _ in 0..len {
            set.insert(T::decode(&mut buf)?);
        }
        Ok(set)
    }
}

impl<B: BufMut, T> Encode<B> for BTreeSet<T>
where
    T: for<'a> Encode<&'a mut B>,
{
    fn encode(&self, mut buf: B) -> Result<(), BoxedError<'static>> {
        buf.put_variable(self.len() as u32);
        for item in self {
            item.encode(&mut buf)?;
        }
        Ok(())
    }
}

impl<'de, B: Buf, T> Decode<'de, B> for BTreeSet<T>
where
    T: for<'a> Decode<'de, &'a mut B> + Ord,
{
    fn decode(mut buf: B) -> Result<Self, BoxedError<'de>> {
        let len = buf.get_variable::<u32>() as usize;
        let mut set = BTreeSet::new();
        for _ in 0..len {
            set.insert(T::decode(&mut buf)?);
        }
        Ok(set)
    }
}

/// A length-prepended cell for encoding and decoding.
#[derive(Debug, Clone, Copy)]
pub struct LengthPrepended<T: ?Sized> {
    max_len: usize,
    inner: T,
}

impl<T> LengthPrepended<T> {
    /// Creates a new length-prepended value for encoding or seed for decoding.
    #[inline]
    pub const fn new(inner: T, max_len: usize) -> Self {
        Self { inner, max_len }
    }

    /// Gets the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> LengthPrepended<PhantomData<T>> {
    /// Creates a new length-prepended seed for decoding without internal seed.
    #[inline]
    pub const fn new_unseeded(max_len: usize) -> Self {
        Self {
            inner: PhantomData,
            max_len,
        }
    }
}

impl<T: ?Sized> LengthPrepended<T> {
    /// Gets the maximum length of encoded bytes.
    #[inline]
    pub fn max_len(&self) -> usize {
        self.max_len
    }
}

impl<T> From<T> for LengthPrepended<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Self::new(inner, usize::MAX)
    }
}

impl<T> Default for LengthPrepended<PhantomData<T>> {
    fn default() -> Self {
        Self::new_unseeded(usize::MAX)
    }
}

impl<T, B> Encode<B> for LengthPrepended<T>
where
    B: BufMut,
    T: for<'b> Encode<&'b mut BytesMut>,
{
    fn encode(&self, mut buf: B) -> Result<(), BoxedError<'static>> {
        let mut bytes = BytesMut::new();
        self.inner.encode(&mut bytes)?;
        // why not 'limit' cells: they panic.
        if bytes.len() > self.max_len {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "length too long: expected <= {} bytes, received {} bytes",
                    self.max_len,
                    bytes.len()
                ),
            )
            .into());
        }
        buf.put_variable(bytes.len() as u32);
        buf.put_slice(&bytes);
        Ok(())
    }
}

impl<'de, T, B> DecodeSeed<'de, B> for LengthPrepended<T>
where
    T: DecodeSeed<'de, bytes::buf::Take<B>>,
    B: Buf,
{
    type Output = T::Output;

    fn decode(self, mut buf: B) -> Result<Self::Output, BoxedError<'de>> {
        let len = buf.get_variable::<u32>() as usize;
        if len > self.max_len {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "length too long: expected <= {} bytes, received {len} bytes",
                    self.max_len
                ),
            )
            .into());
        }
        let taken = buf.take(len);
        self.inner.decode(taken)
    }
}

macro_rules! tuple_impl {
    ($($t:ident),*$(,)?) => {
        impl<B,$($t: for<'env> Encode<&'env mut B>),*> Encode<B> for ($($t,)*) {
            #[allow(non_snake_case)]
            fn encode(&self, mut _buf: B) -> Result<(), BoxedError<'static>> {
                let ($($t,)*) = self;
                $($t.encode(&mut _buf)?;)*
                Ok(())
            }
        }

        impl<'de, B,$($t: for<'env> Decode<'de, &'env mut B>),*> Decode<'de, B> for ($($t,)*) {
            fn decode(mut _buf: B) -> Result<Self, BoxedError<'de>> {
                Ok(($($t::decode(&mut _buf)?,)*))
            }
        }
    };
}

// There exist tuple codec with up to 11 elements in vanilla Minecraft
tuple_impl![];
tuple_impl![T1];
tuple_impl![T1, T2];
tuple_impl![T1, T2, T3];
tuple_impl![T1, T2, T3, T4];
tuple_impl![T1, T2, T3, T4, T5];
tuple_impl![T1, T2, T3, T4, T5, T6];
tuple_impl![T1, T2, T3, T4, T5, T6, T7];
tuple_impl![T1, T2, T3, T4, T5, T6, T7, T8];
tuple_impl![T1, T2, T3, T4, T5, T6, T7, T8, T9];
tuple_impl![T1, T2, T3, T4, T5, T6, T7, T8, T9, T10];
tuple_impl![T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11];
