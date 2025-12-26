//! Downcastable cells and non-static downcasting for Rust.

#![deprecated = "use rcutil instead"]
#![no_std]

use core::{
    any::TypeId,
    ops::{Deref, DerefMut},
};

/// A cell that is able to downcast into a concrete type.
#[derive(Debug, Clone, Copy)]
pub struct Downcast<T: ?Sized> {
    ty: TypeId,
    value: T,
}

/// Trait for types that can be converted into a static variant.
///
/// # Safety
///
/// The repr type should be the static variant of the implemented type.
pub unsafe trait ToStatic {
    /// The static variant of this type.
    type StaticRepr: 'static;
}

impl<T: ToStatic> Downcast<T> {
    /// Creates a new `Downcast` cell.
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            ty: TypeId::of::<T::StaticRepr>(),
            value,
        }
    }
}

impl<T> Downcast<T> {
    /// Creates a new `Downcast` cell with given [`TypeId`].
    ///
    /// # Safety
    ///
    /// This function could not make sure the type id is correct.
    #[inline]
    pub const unsafe fn with_type_id(ty: TypeId, value: T) -> Self {
        Self { ty, value }
    }
}

impl<T> From<T> for Downcast<T>
where
    T: ToStatic,
{
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Default for Downcast<T>
where
    T: Default + ToStatic,
{
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: ?Sized> Downcast<T> {
    /// Downcasts the value into a concrete type, returning an immutable reference.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it could not make sure the lifetime is safe.
    #[inline]
    pub unsafe fn downcast_ref<V: ToStatic>(&self) -> Option<&V> {
        if self.is_safe::<V>() {
            unsafe { Some(&*(core::ptr::from_ref::<T>(&self.value) as *const V)) }
        } else {
            None
        }
    }

    /// Downcasts the value into a concrete type, returning a mutable reference.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it could not make sure the lifetime is safe.
    #[inline]
    pub unsafe fn downcast_mut<V: ToStatic>(&mut self) -> Option<&mut V> {
        if self.is_safe::<V>() {
            unsafe { Some(&mut *(core::ptr::from_mut::<T>(&mut self.value) as *mut V)) }
        } else {
            None
        }
    }

    /// Whether it's safe to downcast into the given concrete type.
    #[inline]
    pub fn is_safe<V: ToStatic>(&self) -> bool {
        self.ty == TypeId::of::<V::StaticRepr>()
    }
}

impl<T: ?Sized> Deref for Downcast<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: ?Sized> DerefMut for Downcast<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// Safe variant of [`ToStatic`] for implementing [`ToStatic`] on
/// static types without unsafe code.
pub trait ToStaticSafe: 'static {}

unsafe impl<T> ToStatic for T
where
    T: ToStaticSafe,
{
    type StaticRepr = Self;
}
