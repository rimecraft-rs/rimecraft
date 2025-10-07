//! Local context traits.

use std::fmt::Debug;

use global_cx::GlobalContext;

pub mod dyn_cx;

pub mod dsyn;
pub mod dyn_codecs;
mod edcode;
pub mod nbt;
pub mod serde;

#[doc(hidden)]
#[cfg(feature = "dsyn")]
pub use ::dsyn as __dsyn;

/// A base local context.
pub trait BaseLocalContext: Sized + Copy {}

/// A local context provides data to the global context.
pub trait LocalContext<T>: BaseLocalContext {
    /// Acquire the data from the local context.
    fn acquire(self) -> T;
}

/// A local context that can be peeked.
pub trait PeekLocalContext<T>: BaseLocalContext {
    /// Peek the data from the local context.
    fn peek_acquire<F, U>(self, f: F) -> U
    where
        F: FnOnce(&T) -> U;
}

impl<'a, L, T: 'a> PeekLocalContext<T> for L
where
    L: LocalContext<&'a T>,
{
    #[inline(always)]
    fn peek_acquire<F, U>(self, f: F) -> U
    where
        F: FnOnce(&T) -> U,
    {
        f(self.acquire())
    }
}

/// A general type that provides explicit local context type.
pub trait ProvideLocalCxTy {
    /// The local context type.
    type LocalContext<'cx>: BaseLocalContext;
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

impl<T, L: BaseLocalContext> ForwardToWithLocalCx for WithLocalCx<T, L> {
    type Forwarded = T;

    type LocalCx = L;

    #[inline]
    fn forward(self) -> WithLocalCx<Self::Forwarded, Self::LocalCx> {
        self
    }
}

impl<'a, T, L: BaseLocalContext> ForwardToWithLocalCx for &'a WithLocalCx<T, L> {
    type Forwarded = &'a T;

    type LocalCx = L;

    #[inline]
    fn forward(self) -> WithLocalCx<Self::Forwarded, Self::LocalCx> {
        WithLocalCx {
            local_cx: self.local_cx,
            inner: &self.inner,
        }
    }
}

impl<'a, T, L: BaseLocalContext> ForwardToWithLocalCx for &'a mut WithLocalCx<T, L> {
    type Forwarded = &'a mut T;

    type LocalCx = L;

    #[inline]
    fn forward(self) -> WithLocalCx<Self::Forwarded, Self::LocalCx> {
        WithLocalCx {
            local_cx: self.local_cx,
            inner: &mut self.inner,
        }
    }
}

impl<'b, T> ForwardToWithLocalCx for &'_ &'b T
where
    &'b T: ForwardToWithLocalCx,
{
    type Forwarded = <&'b T as ForwardToWithLocalCx>::Forwarded;

    type LocalCx = <&'b T as ForwardToWithLocalCx>::LocalCx;

    #[inline]
    fn forward(self) -> WithLocalCx<Self::Forwarded, Self::LocalCx> {
        (*self).forward()
    }
}

impl<'a, T> ForwardToWithLocalCx for &'a mut &'_ mut T
where
    &'a mut T: ForwardToWithLocalCx,
{
    type Forwarded = <&'a mut T as ForwardToWithLocalCx>::Forwarded;

    type LocalCx = <&'a mut T as ForwardToWithLocalCx>::LocalCx;

    #[inline]
    fn forward(self) -> WithLocalCx<Self::Forwarded, Self::LocalCx> {
        (*self).forward()
    }
}

mod tests;
