//! Rimecraft Entity primitives.

use std::fmt::Debug;

use dsyn::HoldDescriptors;
use global_cx::ProvideIdTy;
use registry::Reg;

/// A type of [`Entity`].
pub trait RawEntityType<'a, Cx>: HoldDescriptors<'static, 'a> {
    /// Data type of the target entity.
    type Data;
}

/// [`RawEntityType`] with type erased.
#[allow(missing_docs)]
pub trait ErasedRawEntityType<'a, Cx>: HoldDescriptors<'static, 'a> + Debug {}

impl<'a, Cx, T> ErasedRawEntityType<'a, Cx> for T where T: RawEntityType<'a, Cx> + Debug {}

/// A type of [`Entity`] that can be used in a type erased context.
pub type DynErasedRawEntityType<'r, Cx> = Box<dyn ErasedRawEntityType<'r, Cx> + Send + Sync + 'r>;

/// A type of [`Entity`].
pub type EntityType<'r, Cx> = Reg<'r, <Cx as ProvideIdTy>::Id, DynErasedRawEntityType<'r, Cx>>;

/// An object in a world with double-precision position.
pub struct RawEntity<'a, T: ?Sized, Cx>
where
    Cx: ProvideIdTy,
{
    ty: EntityType<'a, Cx>,

    data: T,
}

impl<T: ?Sized, Cx> Debug for RawEntity<'_, T, Cx>
where
    Cx: ProvideIdTy<Id: Debug>,
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawEntity")
            .field("ty", &self.ty)
            .field("data", &&self.data)
            .finish()
    }
}
