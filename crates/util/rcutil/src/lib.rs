//! General purpose utilities for Rust programming.

#![no_std]

use core::{any::TypeId, marker::PhantomData, mem::MaybeUninit, ptr};

/// Cast a value to itself by checking the type.
///
/// See [`try_cast`] for handling type mismatch.
///
/// # Safety
///
/// This function is unsafe because it may change the **lifetime** of the type.
///
/// # Panics
///
/// Panics if the given types are not equal regardless of their lifetime.
#[inline]
pub unsafe fn cast<L, R>(value: L) -> R {
    assert_eq!(
        typeid::<L>(),
        typeid::<R>(),
        "type mismatch between given types"
    );

    unsafe { transmute::<L, R>(value) }
}

/// Try to cast a value to itself by checking the type.
///
/// # Safety
///
/// This function is unsafe because it may change the **lifetime** of the type.
///
/// # Errors
///
/// Returns the original value if the types are not equal.
#[inline]
pub unsafe fn try_cast<L, R>(value: L) -> Result<R, L> {
    if typeid::<L>() == typeid::<R>() {
        Ok(unsafe { transmute::<L, R>(value) })
    } else {
        Err(value)
    }
}

/// Transmutes a value to another type.
///
/// This has looser checks of memory layout compared to [`core::mem::transmute`], but still guarantees
/// soundness of memory layout.
///
/// # Safety
///
/// This function is unsafe because it transmutes a value to another type.
#[inline]
pub const unsafe fn transmute<L, R>(value: L) -> R {
    struct __TypeCheck<L, R>(PhantomData<(L, R)>);

    impl<L, R> __TypeCheck<L, R> {
        const __CHECK_SIZE: () = assert!(
            size_of::<L>() == size_of::<R>(),
            "size mismatch between types"
        );

        const __ALIGN_SIZE: () = assert!(
            align_of::<L>() == align_of::<R>(),
            "size mismatch between types"
        );
    }

    let _: () = __TypeCheck::<L, R>::__CHECK_SIZE;
    let _: () = __TypeCheck::<L, R>::__ALIGN_SIZE;

    unsafe { transmute_unchecked::<L, R>(value) }
}

/// Transmutes a value to another type with no type checks.
///
/// # Safety
///
/// This function is unsafe because it transmutes a value to another type.
#[inline]
pub const unsafe fn transmute_unchecked<L, R>(value: L) -> R {
    let mut r = MaybeUninit::<R>::uninit();
    unsafe { &mut *ptr::from_mut(&mut r).cast::<MaybeUninit<L>>() }.write(value);
    //SAFETY: we have already written to the uninit memory
    unsafe { r.assume_init() }
}

/// Gets the [`TypeId`] of a type regardless of its lifetime.
#[inline]
pub fn typeid<T: ?Sized>() -> TypeId {
    typeid::of::<T>()
}
