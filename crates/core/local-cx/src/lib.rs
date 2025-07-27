//! Local context traits.

use std::fmt::Debug;

use global_cx::GlobalContext;

pub mod dyn_cx;

pub mod dyn_codecs;
pub mod edcode;
pub mod nbt;
pub mod serde;

/// A base local context.
pub trait BaseLocalContext: Sized + Copy {}

/// A local context provides data to the global context.
pub trait LocalContext<T>: BaseLocalContext {
    /// Acquire the data from the local context.
    fn acquire(self) -> T;
}

/// A general type that provides explicit local context type.
pub trait ProvideLocalCxTy {
    /// The local context type.
    type Context<'cx>: BaseLocalContext;
}

/// Global context types that provides explicit local context type.
///
/// See [`ProvideLocalCxTy`].
pub trait GlobalProvideLocalCxTy: ProvideLocalCxTy + GlobalContext {}

/// A type that carries a local context.
///
/// This type is used to carry a local context along with the data.
pub struct WithLocalCx<T: ?Sized, LocalCx> {
    /// The local context.
    pub local_cx: LocalCx,
    /// The data.
    pub inner: T,
}

impl<T: ?Sized, Cx> WithLocalCx<T, Cx>
where
    Cx: BaseLocalContext,
{
    /// Borrows the inner data.
    #[inline]
    pub fn as_mut(&mut self) -> WithLocalCx<&mut T, Cx> {
        WithLocalCx {
            inner: &mut self.inner,
            local_cx: self.local_cx,
        }
    }
}

impl<T, Cx> Debug for WithLocalCx<T, Cx>
where
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self.inner)
    }
}

/// Extension trait for local context.
pub trait LocalContextExt {
    /// Create a `WithLocalCx` with the given inner data.
    #[inline]
    fn with<T>(self, inner: T) -> WithLocalCx<T, Self>
    where
        Self: Sized,
    {
        WithLocalCx {
            inner,
            local_cx: self,
        }
    }
}

impl<Cx> LocalContextExt for Cx where Cx: BaseLocalContext {}

/// A type that can be transformed into a [`WithLocalCx`] by taking ownership of it.
pub trait ForwardToWithLocalCx {
    /// The type of the inner data.
    type Forwarded;

    /// The type of the local context.
    type LocalCx: BaseLocalContext;

    /// Transform/forward into a [`WithLocalCx`].
    fn forward(self) -> WithLocalCx<Self::Forwarded, Self::LocalCx>;
}

/// A type that can be transformed into a [`WithLocalCx`] by taking reference of it.
pub trait ForwardToWithLocalCxRef {
    /// The type of the inner data.
    type ForwardedRef<'a>
    where
        Self: 'a;

    /// The type of the local context.
    type LocalCx: BaseLocalContext;

    /// Transform/forward into a [`WithLocalCx`] by taking reference.
    fn forward_ref(&self) -> WithLocalCx<Self::ForwardedRef<'_>, Self::LocalCx>;
}

/// A type that can be transformed into a [`WithLocalCx`] by taking mutable reference of it.
pub trait ForwardToWithLocalCxMut: ForwardToWithLocalCxRef {
    /// The type of the inner data.
    type ForwardedMut<'a>
    where
        Self: 'a;

    /// Transform/forward into a [`WithLocalCx`] by taking mutable reference.
    fn forward_mut(&mut self) -> WithLocalCx<Self::ForwardedMut<'_>, Self::LocalCx>;
}

impl<T, L: BaseLocalContext> ForwardToWithLocalCx for WithLocalCx<T, L> {
    type Forwarded = T;

    type LocalCx = L;

    #[inline]
    fn forward(self) -> WithLocalCx<Self::Forwarded, Self::LocalCx> {
        self
    }
}

impl<T: ?Sized, L: BaseLocalContext> ForwardToWithLocalCxRef for WithLocalCx<T, L> {
    type ForwardedRef<'a>
        = &'a T
    where
        Self: 'a;

    type LocalCx = L;

    #[inline]
    fn forward_ref(&self) -> WithLocalCx<Self::ForwardedRef<'_>, Self::LocalCx> {
        WithLocalCx {
            local_cx: self.local_cx,
            inner: &self.inner,
        }
    }
}

impl<T: ?Sized, L: BaseLocalContext> ForwardToWithLocalCxMut for WithLocalCx<T, L> {
    type ForwardedMut<'a>
        = &'a mut T
    where
        Self: 'a;

    #[inline]
    fn forward_mut(&mut self) -> WithLocalCx<Self::ForwardedMut<'_>, Self::LocalCx> {
        WithLocalCx {
            local_cx: self.local_cx,
            inner: &mut self.inner,
        }
    }
}

impl<'borrow, T: ForwardToWithLocalCxRef> ForwardToWithLocalCxRef for &'borrow T {
    type ForwardedRef<'a>
        = T::ForwardedRef<'borrow>
    where
        Self: 'a;

    type LocalCx = T::LocalCx;

    #[inline]
    fn forward_ref(&self) -> WithLocalCx<Self::ForwardedRef<'_>, Self::LocalCx> {
        (**self).forward_ref()
    }
}

impl<T: ForwardToWithLocalCxRef> ForwardToWithLocalCxRef for &'_ mut T {
    type ForwardedRef<'a>
        = T::ForwardedRef<'a>
    where
        Self: 'a;

    type LocalCx = T::LocalCx;

    #[inline]
    fn forward_ref(&self) -> WithLocalCx<Self::ForwardedRef<'_>, Self::LocalCx> {
        (**self).forward_ref()
    }
}

impl<T: ForwardToWithLocalCxMut> ForwardToWithLocalCxMut for &'_ mut T {
    type ForwardedMut<'a>
        = T::ForwardedMut<'a>
    where
        Self: 'a;

    #[inline]
    fn forward_mut(&mut self) -> WithLocalCx<Self::ForwardedMut<'_>, Self::LocalCx> {
        (**self).forward_mut()
    }
}

mod tests;
