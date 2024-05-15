//! Utilities for encoding and decoding common types.

use bytes::{Buf, BufMut};

use crate::{BoxedError, Decode, Encode};

/// A variable-length type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable<T>(pub T);

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
                    if shift > <$t>::BITS {
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
    u8, u16, u32, u64, u128,
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
    i8 => u8,
    i16 => u16,
    i32 => u32,
    i64 => u64,
    i128 => u128,
}
