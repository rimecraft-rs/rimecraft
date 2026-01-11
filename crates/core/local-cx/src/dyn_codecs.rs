//! `serde`, `edcode` codecs on top of type-erasure and
//! dynamic local contexts.

#![cfg(feature = "dyn-codecs")]

use std::{fmt::Debug, marker::PhantomData};

use crate::WithLocalCx;

use rcutil::Any;

#[doc(hidden)]
pub use edcode2::{Buf, BufMut, Decode, Encode};
#[doc(hidden)]
pub use erased_serde::Serialize;

/// Downcast boxed [`Any`] type with type checked.
///
/// # Safety
///
/// Lifetime not guaranteed.
#[allow(clippy::missing_errors_doc)]
pub unsafe fn downcast_boxed<'a, T>(any: Box<dyn Any + 'a>) -> Result<Box<T>, Box<dyn Any + 'a>> {
    if (*any).type_id_dyn() == typeid::of::<T>() {
        unsafe { Ok(Box::from_raw(Box::into_raw(any) as *mut T)) }
    } else {
        Err(any)
    }
}

/// Codec for serialization and deserialization.
pub struct SerdeCodec<T, L, DynS: ?Sized, Dyn: ?Sized = DynS> {
    /// Underlying unsafe codec.
    pub codec: UnsafeSerdeCodec<L, DynS, Dyn>,
    #[doc(hidden)]
    pub _marker: PhantomData<T>,
}

/// Unsafe veriant of [`SerdeCodec`].
pub struct UnsafeSerdeCodec<L, DynS: ?Sized, Dyn: ?Sized = DynS> {
    /// Serialize function.
    pub ser: for<'s, 'o> fn(&'s WithLocalCx<&'o Dyn, L>) -> &'s (dyn Serialize + 'o),
    /// Deserialize function.
    pub de:
        fn(&mut (dyn erased_serde::Deserializer<'_> + '_), L) -> erased_serde::Result<Box<DynS>>,
}

/// Codec for packet encoding and decoding.
pub struct EdcodeCodec<T, L, DynS: ?Sized, Dyn: ?Sized = DynS> {
    /// Underlying unsafe codec.
    pub codec: UnsafeEdcodeCodec<L, DynS, Dyn>,
    #[doc(hidden)]
    pub _marker: PhantomData<T>,
}

/// Unsafe variant of [`EdcodeCodec`].
pub struct UnsafeEdcodeCodec<L, DynS: ?Sized, Dyn: ?Sized = DynS> {
    /// Encode function.
    pub encode:
        fn(&'_ Dyn, &'_ mut (dyn BufMut + '_), L) -> Result<(), edcode2::BoxedError<'static>>,
    /// Decode function.
    pub decode: fn(&'_ mut (dyn Buf + '_), L) -> Result<Box<DynS>, edcode2::BoxedError<'static>>,
}

/// Generates a [`SerdeCodec`].
///
/// Syntax: `Type: Trait + 'lifetime`
#[macro_export]
macro_rules! serde_codec {
    (Local<$lcx:ty>$t:ty:$tr:ident+$l:lifetime) => {
        $crate::dyn_codecs::SerdeCodec {
            codec: $crate::dyn_codecs::UnsafeSerdeCodec {
                ser: |obj| unsafe {
                    &*(::std::ptr::from_ref::<$crate::WithLocalCx<&(dyn $tr + '_), $lcx>>(obj)
                        as *const $crate::WithLocalCx<&$t, $lcx>
                        as *const (dyn $crate::dyn_codecs::Serialize + $l))
                },
                de: |deserializer, cx| {
                    <$t as $crate::serde::DeserializeWithCx<'_, _>>::deserialize_with_cx(
                        $crate::LocalContextExt::with(cx, deserializer),
                    )
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
/// Syntax: `(Nbt<Cx>)or(Local<LocalCx>) Type: Trait + 'lifetime`
#[macro_export]
macro_rules! edcode_codec {
    (Local<$lcx:ty>$t:ty:$tr:ident+$l:lifetime) => {
        $crate::dyn_codecs::EdcodeCodec {
            codec: $crate::dyn_codecs::UnsafeEdcodeCodec {
                encode: |obj, buf, cx| {
                    $crate::dyn_codecs::Encode::encode(
                        unsafe { &*(::std::ptr::from_ref::<dyn $tr + '_>(obj) as *const $t) },
                        $crate::LocalContextExt::with(cx, buf),
                    )
                },
                decode: {
                    ::std::assert!(
                        <$t as $crate::dyn_codecs::Decode<
                            '_,
                            $crate::WithLocalCx<&'_ mut dyn $crate::dyn_codecs::Buf, $lcx>,
                        >>::SUPPORT_NON_IN_PLACE,
                        "non-in-place decoding is not supported for this type",
                    );
                    |buf, cx| {
                        ::std::result::Result::Ok(::std::boxed::Box::new(
                            <$t as $crate::dyn_codecs::Decode<'_, _>>::decode(
                                $crate::LocalContextExt::with(cx, buf),
                            )?,
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
                    <$cx as $crate::nbt::WriteNbtWithCx<&'_ $t, _>>::write_nbt(
                        unsafe { &*(::std::ptr::from_ref::<dyn $tr + '_>(obj) as *const $t) },
                        $crate::LocalContextExt::with(cx, $crate::dyn_codecs::BufMut::writer(buf)),
                    )
                    .map_err(::std::convert::Into::into)
                },
                decode: |buf, cx| {
                    Ok(::std::boxed::Box::new(
                        <$cx as $crate::nbt::ReadNbtWithCx<$t, _>>::read_nbt(
                            $crate::LocalContextExt::with(cx, $crate::dyn_codecs::Buf::reader(buf)),
                        )?,
                    ))
                },
            },
            _marker: ::std::marker::PhantomData,
        }
    };
}

impl<L, A: ?Sized, B: ?Sized> Debug for UnsafeSerdeCodec<L, A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnsafeSerdeCodec").finish()
    }
}
impl<L, A: ?Sized, B: ?Sized> Debug for UnsafeEdcodeCodec<L, A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnsafeEdcodeCodec").finish()
    }
}
impl<T, L, A: ?Sized, B: ?Sized> Debug for SerdeCodec<T, L, A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerdeCodec").finish()
    }
}
impl<T, L, A: ?Sized, B: ?Sized> Debug for EdcodeCodec<T, L, A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EdcodeCodec").finish()
    }
}
impl<L, A: ?Sized, B: ?Sized> Copy for UnsafeSerdeCodec<L, A, B> {}
impl<L, A: ?Sized, B: ?Sized> Copy for UnsafeEdcodeCodec<L, A, B> {}
impl<T, L, A: ?Sized, B: ?Sized> Copy for SerdeCodec<T, L, A, B> {}
impl<T, L, A: ?Sized, B: ?Sized> Copy for EdcodeCodec<T, L, A, B> {}
impl<L, A: ?Sized, B: ?Sized> Clone for UnsafeSerdeCodec<L, A, B> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<L, A: ?Sized, B: ?Sized> Clone for UnsafeEdcodeCodec<L, A, B> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, L, A: ?Sized, B: ?Sized> Clone for SerdeCodec<T, L, A, B> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, L, A: ?Sized, B: ?Sized> Clone for EdcodeCodec<T, L, A, B> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
