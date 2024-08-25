//! `edcode2` crate integration.

use core::marker::PhantomData;

use edcode2::{Buf, BufMut, Decode, Encode};

use crate::nbt::{ReadNbt, WriteNbt};

/// A type that wraps a value with a context.
#[derive(Debug)]
pub struct Nbt<T, Cx>(pub T, PhantomData<Cx>);

impl<T, Cx> Nbt<T, Cx> {
    /// Creates a new `Nbt` with the given value.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(value, PhantomData)
    }

    /// Consumes the `Nbt` and returns the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, Cx> From<T> for Nbt<T, Cx> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<B, T, Cx> Encode<B> for Nbt<T, Cx>
where
    B: BufMut,
    Cx: for<'s> WriteNbt<&'s T>,
{
    #[inline]
    fn encode(&self, buf: B) -> Result<(), edcode2::BoxedError<'static>> {
        Cx::write_nbt(&self.0, buf.writer()).map_err(Into::into)
    }
}

impl<'de, B, T, Cx> Decode<'de, B> for Nbt<T, Cx>
where
    B: Buf,
    Cx: ReadNbt<T>,
{
    #[inline]
    fn decode(buf: B) -> Result<Self, edcode2::BoxedError<'de>> {
        Cx::read_nbt(buf.reader())
            .map(Self::new)
            .map_err(Into::into)
    }
}
