//! Local context traits.

use std::fmt::Debug;

use global_cx::GlobalContext;

pub mod dyn_cx;

mod edcode;
pub mod serde;

/// A base local context.
pub trait BaseLocalContext: Sized + Copy {}

/// A local context provides data to the global context.
pub trait LocalContext<T>: BaseLocalContext {
    /// Acquire the data from the local context.
    fn acquire(self) -> T;
}

/// Global context types that provides implicit local context type.
pub trait ProvideLocalCxTy: GlobalContext {
    /// The local context type.
    type Context<'cx>: BaseLocalContext;
}

/// A type that carries a local context.
///
/// This type is used to carry a local context along with the data.
pub struct WithLocalCx<T, LocalCx> {
    /// The data.
    pub inner: T,
    /// The local context.
    pub local_cx: LocalCx,
}

impl<T, Cx> Debug for WithLocalCx<T, Cx>
where
    T: Debug,
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
