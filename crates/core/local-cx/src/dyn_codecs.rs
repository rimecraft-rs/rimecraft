//! `serde`, `edcode` codecs on top of type-erasure and
//! dynamic local contexts.

#![cfg(feature = "dyn-codecs")]

use std::{any::TypeId, marker::PhantomData};

use edcode2::{Buf, BufMut, Decode, Encode};

use crate::{
    LocalContextExt, WithLocalCx,
    dyn_cx::UnsafeDynamicContext,
    serde::{DeserializeWithCx, SerializeWithCx},
};

/// An 'any' trait without any type restriction.
pub trait Any {
    /// Gets the [`TypeId`] of this type.
    #[inline(always)]
    fn type_id(&self) -> TypeId {
        typeid::of::<Self>()
    }
}

impl<T: ?Sized> Any for T {}

impl dyn Any + Send + Sync + '_ {
    /// Downcast [`Any`] type with type checked.
    ///
    /// # Safety
    ///
    /// Lifetime not guaranteed.
    pub unsafe fn downcast_ref<T>(&self) -> Option<&T> {
        if (*self).type_id() == typeid::of::<T>() {
            unsafe { Some(&*(std::ptr::from_ref::<dyn Any + Send + Sync + '_>(self) as *const T)) }
        } else {
            None
        }
    }

    /// Downcast [`Any`] type mutably with type checked.
    ///
    /// # Safety
    ///
    /// Lifetime not guaranteed.
    pub unsafe fn downcast_mut<T>(&mut self) -> Option<&mut T> {
        if (*self).type_id() == typeid::of::<T>() {
            unsafe {
                Some(&mut *(std::ptr::from_mut::<dyn Any + Send + Sync + '_>(self) as *mut T))
            }
        } else {
            None
        }
    }
}

/// Downcast boxed [`Any`] type with type checked.
///
/// # Safety
///
/// Lifetime not guaranteed.
pub unsafe fn downcast_boxed<'a, T>(
    any: Box<dyn Any + Send + Sync + 'a>,
) -> Result<Box<T>, Box<dyn Any + Send + Sync + 'a>> {
    if (*any).type_id() == typeid::of::<T>() {
        unsafe { Ok(Box::from_raw(Box::into_raw(any) as *mut T)) }
    } else {
        Err(any)
    }
}

type Object<'a> = dyn Any + Send + Sync + 'a;

/// Codec for serialization and deserialization.
#[derive(Debug, Clone, Copy)]
pub struct SerdeCodec<'a, T> {
    codec: UnsafeSerdeCodec<'a>,
    _marker: PhantomData<T>,
}

/// Unsafe veriant of [`SerdeCodec`].
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct UnsafeSerdeCodec<'a> {
    /// Serialize function.
    pub ser: for<'s, 'o> fn(
        &'s WithLocalCx<&'o Object<'a>, UnsafeDynamicContext<'_>>,
    ) -> &'s (dyn erased_serde::Serialize + 'o),
    /// Deserialize function.
    pub de: fn(
        &mut dyn erased_serde::Deserializer<'_>,
        UnsafeDynamicContext<'_>,
    ) -> erased_serde::Result<Box<Object<'a>>>,
}

/// Codec for packet encoding and decoding.
#[derive(Debug, Clone, Copy)]
pub struct EdcodeCodec<'a, T> {
    codec: UnsafeEdcodeCodec<'a>,
    _marker: PhantomData<T>,
}

/// Unsafe variant of [`EdcodeCodec`].
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct UnsafeEdcodeCodec<'a> {
    /// Encode function.
    pub encode: fn(
        &'_ Object<'a>,
        &'_ mut dyn BufMut,
        UnsafeDynamicContext<'_>,
    ) -> Result<(), edcode2::BoxedError<'static>>,
    /// Decode function.
    pub decode: fn(
        &'_ mut dyn Buf,
        UnsafeDynamicContext<'_>,
    ) -> Result<Box<Object<'a>>, edcode2::BoxedError<'static>>,
}

impl<'a, T> SerdeCodec<'a, T>
where
    for<'t, 'cx> &'t T: SerializeWithCx<UnsafeDynamicContext<'cx>>,
    T: for<'de, 'cx> DeserializeWithCx<'de, UnsafeDynamicContext<'cx>> + Send + Sync + 'a,
{
    /// Creates a new [`SerdeCodec`] by using `erased_serde`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            codec: UnsafeSerdeCodec {
                ser: |obj| unsafe {
                    &*(std::ptr::from_ref::<WithLocalCx<&Object<'_>, UnsafeDynamicContext<'_>>>(obj)
                        as *const WithLocalCx<&T, UnsafeDynamicContext<'_>>
                        as *const (dyn erased_serde::Serialize + 'a))
                },
                de: |deserializer, cx| {
                    T::deserialize_with_cx(cx.with(deserializer)).map(|v| {
                        let v: Box<Object<'_>> = Box::new(v);
                        v
                    })
                },
            },
            _marker: PhantomData,
        }
    }
}

impl<'a, T> SerdeCodec<'a, T> {
    /// Gets the unsafe variant of this codec.
    #[inline]
    pub const fn to_unsafe(self) -> UnsafeSerdeCodec<'a> {
        self.codec
    }
}

impl<'a, T> EdcodeCodec<'a, T> {
    /// [`Self::new`] but wrapped by two helper types.
    pub const fn new_wrapped<E, D>() -> Self
    where
        D: Into<T>,
        &'a T: Into<E>,
        E: for<'b, 'cx> Encode<WithLocalCx<&'b mut dyn BufMut, UnsafeDynamicContext<'cx>>>,
        D: for<'b, 'cx> Decode<'static, WithLocalCx<&'b mut dyn Buf, UnsafeDynamicContext<'cx>>>,
        T: Send + Sync + 'a,
    {
        Self {
            codec: UnsafeEdcodeCodec {
                encode: |obj, buf, cx| {
                    Into::<E>::into(unsafe {
                        &*(std::ptr::from_ref::<Object<'_>>(obj) as *const T)
                    })
                    .encode(cx.with(buf))
                },
                decode: {
                    assert!(
                        <D as Decode<'_, _>>::SUPPORT_NON_IN_PLACE,
                        "non-in-place decoding is not supported for this type",
                    );
                    |buf, cx| Ok(Box::new(D::decode(cx.with(buf))?.into()))
                },
            },
            _marker: PhantomData,
        }
    }

    /// Gets the unsafe variant of this codec.
    #[inline]
    pub const fn to_unsafe(self) -> UnsafeEdcodeCodec<'a> {
        self.codec
    }
}

impl<'a, T> EdcodeCodec<'a, T>
where
    T: for<'b, 'cx> Encode<WithLocalCx<&'b mut dyn BufMut, UnsafeDynamicContext<'cx>>>
        + for<'b, 'cx> Decode<'static, WithLocalCx<&'b mut dyn Buf, UnsafeDynamicContext<'cx>>>
        + Send
        + Sync
        + 'a,
{
    /// Creates a new [`EdcodeCodec`] by encoding and decoding through `edcode2`.
    #[inline(always)]
    pub const fn new() -> Self {
        Self::new_wrapped::<&'a T, T>()
    }
}
