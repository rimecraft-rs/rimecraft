//! Local context traits.

use std::fmt::Debug;

use global_cx::GlobalContext;

pub mod dyn_cx;

pub mod dyn_codecs;
mod edcode;
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

mod tests;
