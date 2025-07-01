//! `serde`, `edcode` codecs on top of type-erasure and
//! dynamic local contexts.

#![cfg(feature = "dyn-codecs")]

use std::{any::TypeId, fmt::Debug, marker::PhantomData};

use crate::{WithLocalCx, dyn_cx::UnsafeDynamicContext};

#[doc(hidden)]
pub use edcode2::{Buf, BufMut, Decode, Encode};
#[doc(hidden)]
pub use erased_serde::Serialize;

/// An 'any' trait without any type restriction.
pub trait Any {
    /// Gets the [`TypeId`] of this type.
    #[inline(always)]
    fn type_id(&self) -> TypeId {
        typeid::of::<Self>()
    }
}

impl<T: ?Sized> Any for T {}

impl dyn Any + '_ {
    /// Downcast [`Any`] type with type checked.
    ///
    /// # Safety
    ///
    /// Lifetime not guaranteed.
    pub unsafe fn downcast_ref<T>(&self) -> Option<&T> {
        if (*self).type_id() == typeid::of::<T>() {
            unsafe { Some(&*(std::ptr::from_ref::<dyn Any + '_>(self) as *const T)) }
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
            unsafe { Some(&mut *(std::ptr::from_mut::<dyn Any + '_>(self) as *mut T)) }
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
#[allow(clippy::missing_errors_doc)]
pub unsafe fn downcast_boxed<'a, T>(any: Box<dyn Any + 'a>) -> Result<Box<T>, Box<dyn Any + 'a>> {
    if (*any).type_id() == typeid::of::<T>() {
        unsafe { Ok(Box::from_raw(Box::into_raw(any) as *mut T)) }
    } else {
        Err(any)
    }
}

/// Codec for serialization and deserialization.
pub struct SerdeCodec<T, DynS: ?Sized, Dyn: ?Sized = DynS> {
    /// Underlying unsafe codec.
    pub codec: UnsafeSerdeCodec<DynS, Dyn>,
    #[doc(hidden)]
    pub _marker: PhantomData<T>,
}

/// Unsafe veriant of [`SerdeCodec`].
pub struct UnsafeSerdeCodec<DynS: ?Sized, Dyn: ?Sized = DynS> {
    /// Serialize function.
    pub ser: for<'s, 'o> fn(
        &'s WithLocalCx<&'o Dyn, UnsafeDynamicContext<'_>>,
    ) -> &'s (dyn Serialize + 'o),
    /// Deserialize function.
    pub de: fn(
        &mut dyn erased_serde::Deserializer<'_>,
        UnsafeDynamicContext<'_>,
    ) -> erased_serde::Result<Box<DynS>>,
}

/// Codec for packet encoding and decoding.
pub struct EdcodeCodec<T, DynS: ?Sized, Dyn: ?Sized = DynS> {
    /// Underlying unsafe codec.
    pub codec: UnsafeEdcodeCodec<DynS, Dyn>,
    #[doc(hidden)]
    pub _marker: PhantomData<T>,
}

/// Unsafe variant of [`EdcodeCodec`].
pub struct UnsafeEdcodeCodec<DynS: ?Sized, Dyn: ?Sized = DynS> {
    /// Encode function.
    pub encode: fn(
        &'_ Dyn,
        &'_ mut dyn BufMut,
        UnsafeDynamicContext<'_>,
    ) -> Result<(), edcode2::BoxedError<'static>>,
    /// Decode function.
    pub decode: fn(
        &'_ mut dyn Buf,
        UnsafeDynamicContext<'_>,
    ) -> Result<Box<DynS>, edcode2::BoxedError<'static>>,
}

/// Generates a [`SerdeCodec`].
///
/// Syntax: `Type: Trait + 'lifetime`
#[macro_export]
macro_rules! serde_codec {
    ($t:ty:$tr:ident+$l:lifetime) => {
        $crate::dyn_codecs::SerdeCodec {
            codec: $crate::dyn_codecs::UnsafeSerdeCodec {
                ser: |obj| unsafe {
                    &*(::std::ptr::from_ref::<
                        $crate::WithLocalCx<
                            &(dyn $tr + '_),
                            $crate::dyn_cx::UnsafeDynamicContext<'_>,
                        >,
                    >(obj)
                        as *const $crate::WithLocalCx<&$t, $crate::dyn_cx::UnsafeDynamicContext<'_>>
                        as *const (dyn $crate::dyn_codecs::Serialize + $l))
                },
                de: |deserializer, cx| {
                    <$t as $crate::serde::DeserializeWithCx<
                        '_,
                        $crate::dyn_cx::UnsafeDynamicContext<'_>,
                    >>::deserialize_with_cx($crate::LocalContextExt::with(
                        cx,
                        deserializer,
                    ))
                    .map(|v| {
                        let v: ::std::boxed::Box<
                            dyn $tr + ::std::marker::Send + ::std::marker::Sync + $l,
                        > = ::std::boxed::Box::new(v);
                        v
                    })
                },
            },
            _marker: ::std::marker::PhantomData,
        }
    };
}

/// Generates an [`EdcodeCodec`].
///
/// Syntax: `(?Nbt<Cx>) Type: Trait + 'lifetime`
#[macro_export]
macro_rules! edcode_codec {
    ($t:ty:$tr:ident+$l:lifetime) => {
        $crate::dyn_codecs::EdcodeCodec {
            codec: $crate::dyn_codecs::UnsafeEdcodeCodec {
                encode: |obj, buf, cx| {
                    ::edcode2::Encode::encode(
                        unsafe { &*(::std::ptr::from_ref::<dyn $tr + '_>(obj) as *const $t) },
                        $crate::LocalContextExt::with(cx, buf),
                    )
                },
                decode: {
                    ::std::assert!(
                        <$t as $crate::dyn_codecs::Decode<
                            '_,
                            $crate::WithLocalCx<
                                &'_ mut dyn Buf,
                                $crate::dyn_cx::UnsafeDynamicContext<'_>,
                            >,
                        >>::SUPPORT_NON_IN_PLACE,
                        "non-in-place decoding is not supported for this type",
                    );
                    |buf, cx| {
                        ::std::result::Result::Ok(::std::boxed::Box::new(
                            <$t as $crate::dyn_codecs::Decode<
                                '_,
                                $crate::WithLocalCx<_, $crate::dyn_cx::UnsafeDynamicContext<'_>>,
                            >>::decode($crate::LocalContextExt::with(
                                cx, buf,
                            ))?,
                        ))
                    }
                },
            },
            _marker: ::std::marker::PhantomData,
        }
    };
    (Nbt<$cx:ty>$t:ty:$tr:ident+$l:lifetime) => {
        $crate::dyn_codecs::EdcodeCodec {
            codec: $crate::dyn_codecs::UnsafeEdcodeCodec {
                encode: |obj, buf, cx| {
                    <$cx as $crate::nbt::WriteNbtWithCx<
                        &'_ $t,
                        $crate::dyn_cx::UnsafeDynamicContext<'_>,
                    >>::write_nbt(
                        unsafe { &*(::std::ptr::from_ref::<dyn $tr + '_>(obj) as *const $t) },
                        $crate::LocalContextExt::with(cx, $crate::dyn_codecs::BufMut::writer(buf)),
                    )
                    .map_err(::std::convert::Into::into)
                },
                decode: |buf, cx| {
                    Ok(::std::boxed::Box::new(
                        <$cx as $crate::nbt::ReadNbtWithCx<
                            $t,
                            $crate::dyn_cx::UnsafeDynamicContext<'_>,
                        >>::read_nbt($crate::LocalContextExt::with(
                            cx,
                            $crate::dyn_codecs::Buf::reader(buf),
                        ))?,
                    ))
                },
            },
            _marker: ::std::marker::PhantomData,
        }
    };
}

impl<A: ?Sized, B: ?Sized> Debug for UnsafeSerdeCodec<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnsafeSerdeCodec").finish()
    }
}
impl<A: ?Sized, B: ?Sized> Debug for UnsafeEdcodeCodec<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnsafeEdcodeCodec").finish()
    }
}
impl<T, A: ?Sized, B: ?Sized> Debug for SerdeCodec<T, A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerdeCodec").finish()
    }
}
impl<T, A: ?Sized, B: ?Sized> Debug for EdcodeCodec<T, A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EdcodeCodec").finish()
    }
}
impl<A: ?Sized, B: ?Sized> Copy for UnsafeSerdeCodec<A, B> {}
impl<A: ?Sized, B: ?Sized> Copy for UnsafeEdcodeCodec<A, B> {}
impl<T, A: ?Sized, B: ?Sized> Copy for SerdeCodec<T, A, B> {}
impl<T, A: ?Sized, B: ?Sized> Copy for EdcodeCodec<T, A, B> {}
impl<A: ?Sized, B: ?Sized> Clone for UnsafeSerdeCodec<A, B> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<A: ?Sized, B: ?Sized> Clone for UnsafeEdcodeCodec<A, B> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, A: ?Sized, B: ?Sized> Clone for SerdeCodec<T, A, B> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, A: ?Sized, B: ?Sized> Clone for EdcodeCodec<T, A, B> {
    fn clone(&self) -> Self {
        *self
    }
}
