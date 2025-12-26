//! General purpose utilities for Rust programming.

#![no_std]

use core::{any::TypeId, marker::PhantomData, mem::ManuallyDrop};

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

/// Cast an invariant value to itself by checking the type.
///
/// See [`try_cast_invariant`] for handling type mismatch.
///
/// # Panics
///
/// Panics if the given types are not equal.
#[inline]
pub fn cast_invariant<'a, L, R>(value: L) -> R
where
    L: InvariantOn<'a>,
    R: InvariantOn<'a>,
{
    assert_eq!(
        L::type_id(),
        R::type_id(),
        "type mismatch between given types"
    );

    // SAFETY: L and R are invariant and sharing the same lifetime.
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

/// Try to cast an invariant value to itself by checking the type.
///
/// # Errors
///
/// Returns the original value if the types are not equal.
#[inline]
pub fn try_cast_invariant<'a, L, R>(value: L) -> Result<R, L>
where
    L: InvariantOn<'a>,
    R: InvariantOn<'a>,
{
    if L::type_id() == R::type_id() {
        // SAFETY: L and R are invariant and sharing the same lifetime.
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
    union __UnionCast<L, R> {
        l: ManuallyDrop<L>,
        r: ManuallyDrop<R>,
    }

    let cast = __UnionCast {
        l: ManuallyDrop::new(value),
    };
    let r = unsafe { cast.r };
    ManuallyDrop::into_inner(r)
}

/// Gets the [`TypeId`] of a type regardless of its lifetime.
#[inline]
pub fn typeid<T: ?Sized>() -> TypeId {
    typeid::of::<T>()
}

/// A marker type for invariant lifetime marking.
#[allow(missing_debug_implementations)] // should not have an instance
pub struct InvariantLifetime<'a> {
    _marker: PhantomInvariant<'a>,
}

/// Invariant phantom lifetime.
pub type PhantomInvariant<'a> = PhantomData<fn(&'a ()) -> &'a ()>;

/// Shorthand for creating a [`PhantomInvariant`].
#[inline]
pub const fn phantom_invariant<'a>() -> PhantomInvariant<'a> {
    PhantomData::<fn(&'a ()) -> &'a ()>
}

impl sealed::SealedInvariantLifetime for InvariantLifetime<'_> {}

/// Types that are supposed to be static-lifetimed all the time.
pub trait Static: 'static {}

/// A type that is invariant on its only lifetime.
///
/// # Safety
///
/// The type has to be invariant on its lifetime, and which should be the lifetime
/// referred by [`Self::Lifetime`].
/// The lifetime should also be the only single lifetime of the type.
///
/// After all the purpose of this trait is for safely tranmuting between types,
/// so go with it if you consider it is safe.
///
/// # Safe Implementation
///
/// Implement [`Static`] for your always-static type, while this trait will be
/// automatically implemented for it.
pub unsafe trait Invariant {
    /// The lifetime, with the type of [`InvariantLifetime`].
    type Lifetime: sealed::SealedInvariantLifetime;

    #[doc(hidden)]
    #[inline]
    fn type_id() -> TypeId
    where
        Self: Sized,
    {
        typeid::<Self>()
    }
}

/// Shorthand for `Invariant<Lifetime = InvariantLifetime<'_>'>`.
pub trait InvariantOn<'a>: Invariant<Lifetime = InvariantLifetime<'a>> {}

unsafe impl<T> Invariant for T
where
    T: Static + ?Sized,
{
    type Lifetime = InvariantLifetime<'static>;

    #[inline]
    fn type_id() -> TypeId
    where
        Self: Sized,
    {
        // optimized implementation
        TypeId::of::<Self>()
    }
}

impl<'a, T> InvariantOn<'a> for T where T: Invariant<Lifetime = InvariantLifetime<'a>> + ?Sized {}

mod sealed {
    pub trait SealedInvariantLifetime {}
}
