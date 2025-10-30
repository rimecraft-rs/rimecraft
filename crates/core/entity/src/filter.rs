//! Entity type filters.

use std::{any::TypeId, marker::PhantomData};

use crate::{EntityCx, ErasedData, RawEntity};

/// A filter that filters unsized types into some other subtypes.
pub trait TypeFilter<T: ?Sized> {
    /// The output subtype.
    type Output: ?Sized;

    /// Downcasts the given object into the output subtype.
    ///
    /// # Safety
    ///
    /// - The given raw pointer must be valid to dereference.
    /// - The lifetime in `Self::Output` is not guaranteed to be valid.
    unsafe fn cast_const(&self, obj: *const T) -> Option<*const Self::Output>;

    /// Downcasts the given object into the output subtype.
    ///
    /// # Safety
    ///
    /// - The given raw pointer must be valid to dereference.
    /// - The lifetime in `Self::Output` is not guaranteed to be valid.
    unsafe fn cast_mut(&self, obj: *mut T) -> Option<*mut Self::Output>;

    /// Returns the type id of the output subtype, if known as, and only if is a _concrete type._
    #[inline]
    fn hint_typeid(&self) -> Option<TypeId> {
        None
    }

    /// Guarantees the soundness of the cast.
    ///
    /// # Safety
    ///
    /// The conversion itself, the input-to-output type conversion, must be safe and sound.
    #[inline]
    unsafe fn make_safe(&self) -> impl SafeTypeFilter<T> {
        #[repr(transparent)]
        struct Safe<'a, T: ?Sized>(&'a T);

        impl<T: ?Sized, In: ?Sized> TypeFilter<In> for Safe<'_, T>
        where
            T: TypeFilter<In>,
        {
            type Output = T::Output;

            #[inline]
            unsafe fn cast_const(&self, obj: *const In) -> Option<*const Self::Output> {
                unsafe { self.0.cast_const(obj) }
            }

            #[inline]
            unsafe fn cast_mut(&self, obj: *mut In) -> Option<*mut Self::Output> {
                unsafe { self.0.cast_mut(obj) }
            }

            #[inline]
            fn hint_typeid(&self) -> Option<TypeId> {
                self.0.hint_typeid()
            }
        }

        unsafe impl<T: ?Sized, In: ?Sized> SafeTypeFilter<In> for Safe<'_, T> where T: TypeFilter<In> {}

        Safe(self)
    }
}

/// A [`TypeFilter`] that guarantees the soundness of the cast.
///
/// # Safety
///
/// The conversion itself, the input-to-output type conversion, must be safe and sound.
pub unsafe trait SafeTypeFilter<T: ?Sized>: TypeFilter<T> {}

impl<'a, Cx, T, In: ?Sized> TypeFilter<RawEntity<'a, In, Cx>> for PhantomData<RawEntity<'a, T, Cx>>
where
    Cx: EntityCx<'a>,
    In: ErasedData<'a, Cx>,
{
    type Output = RawEntity<'a, T, Cx>;

    #[inline]
    unsafe fn cast_const(&self, obj: *const RawEntity<'a, In, Cx>) -> Option<*const Self::Output> {
        (unsafe { &*obj }.data().type_id() == typeid::of::<T>()).then_some(obj.cast())
    }

    #[inline]
    unsafe fn cast_mut(&self, obj: *mut RawEntity<'a, In, Cx>) -> Option<*mut Self::Output> {
        (unsafe { &*obj }.data().type_id() == typeid::of::<T>()).then_some(obj.cast())
    }

    #[inline]
    fn hint_typeid(&self) -> Option<TypeId> {
        Some(typeid::of::<RawEntity<'a, T, Cx>>())
    }
}

impl<T: ?Sized> TypeFilter<T> for () {
    type Output = T;

    #[inline]
    unsafe fn cast_const(&self, obj: *const T) -> Option<*const Self::Output> {
        Some(obj)
    }

    #[inline]
    unsafe fn cast_mut(&self, obj: *mut T) -> Option<*mut Self::Output> {
        Some(obj)
    }
}

//SAFETY: identical casts
unsafe impl<T: ?Sized> SafeTypeFilter<T> for () {}
