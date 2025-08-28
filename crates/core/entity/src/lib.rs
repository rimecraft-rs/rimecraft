//! Rimecraft Entity primitives.

use std::{any::TypeId, fmt::Debug};

use dsyn::HoldDescriptors;
use erased_serde::Serialize as ErasedSerialize;
use global_cx::ProvideIdTy;
use registry::Reg;
use serde_update::erased::ErasedUpdate;
use uuid::Uuid;

/// A type of [`Entity`].
pub trait RawEntityType<'a, Cx>: HoldDescriptors<'static, 'a>
where
    Cx: ProvideIdTy,
{
    /// Data type of the target entity.
    type Data: Data<'a, Cx>;

    /// Creates a new entity.
    fn create(&self, this: EntityType<'a, Cx>) -> !; //TODO
}

/// [`RawEntityType`] with type erased.
#[allow(missing_docs)]
pub trait ErasedRawEntityType<'a, Cx>: HoldDescriptors<'static, 'a> + Debug {}

impl<'a, Cx, T> ErasedRawEntityType<'a, Cx> for T
where
    T: RawEntityType<'a, Cx> + Debug,
    Cx: ProvideIdTy,
{
}

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
    uuid: Uuid,

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
            .field("uuid", &self.uuid)
            .field("data", &&self.data)
            .finish()
    }
}

/// A trait for generic entity data types.
pub trait Data<'a, Cx>
where
    Cx: ProvideIdTy,
{
}

/// Type erased entity data.
///
/// See [`Data`].
pub trait ErasedData<'a, Cx>
where
    Self: ErasedSerialize
        + for<'de> ErasedUpdate<'de>
        + Data<'a, Cx>
        + Send
        + Sync
        + Debug
        + sealed::Sealed,
    Cx: ProvideIdTy,
{
    /// The [`TypeId`] of this data.
    #[inline]
    fn type_id(&self) -> TypeId {
        typeid::of::<Self>()
    }
}

#[allow(single_use_lifetimes)]
mod ser_dyn_obj {
    use super::*;
    use erased_serde::serialize_trait_object;
    serialize_trait_object!(<'a, Cx> ErasedData<'a, Cx> where Cx: ProvideIdTy);
}

impl<'de, Cx> serde_update::Update<'de> for dyn ErasedData<'_, Cx> + '_
where
    Cx: ProvideIdTy,
{
    serde_update::__internal_update_from_erased!();
}
impl<'de, Cx> serde_update::Update<'de> for dyn ErasedData<'_, Cx> + Send + '_
where
    Cx: ProvideIdTy,
{
    serde_update::__internal_update_from_erased!();
}
impl<'de, Cx> serde_update::Update<'de> for dyn ErasedData<'_, Cx> + Sync + '_
where
    Cx: ProvideIdTy,
{
    serde_update::__internal_update_from_erased!();
}
impl<'de, Cx> serde_update::Update<'de> for dyn ErasedData<'_, Cx> + Send + Sync + '_
where
    Cx: ProvideIdTy,
{
    serde_update::__internal_update_from_erased!();
}

mod sealed {
    pub trait Sealed {}
}

impl<T> sealed::Sealed for T where T: ErasedSerialize + for<'de> ErasedUpdate<'de> + Send + Sync {}

impl<'a, T, Cx> ErasedData<'a, Cx> for T
where
    T: ErasedSerialize + for<'de> ErasedUpdate<'de> + Data<'a, Cx> + Debug + Send + Sync,
    Cx: ProvideIdTy,
{
}

/// A type-erased variant of [`RawEntity`].
pub type Entity<'w, Cx> = RawEntity<'w, dyn ErasedData<'w, Cx> + 'w, Cx>;

impl<'w, Cx> Entity<'w, Cx>
where
    Cx: ProvideIdTy,
{
    /// Downcasts this type erased entity into entity with a concrete data type.
    ///
    /// This function returns an immutable reference if the type matches.
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'w`.
    pub unsafe fn downcast_ref<T>(&self) -> Option<&RawEntity<'w, T, Cx>> {
        if self.matches_type::<T>() {
            unsafe {
                Some(&*(std::ptr::from_ref::<Entity<'w, Cx>>(self) as *const RawEntity<'w, T, Cx>))
            }
        } else {
            None
        }
    }

    /// Downcasts this type erased entity into entity with a concrete data type.
    ///
    /// This function returns a mutable reference if the type matches.
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'w`.
    pub unsafe fn downcast_mut<T>(&mut self) -> Option<&mut RawEntity<'w, T, Cx>> {
        if self.matches_type::<T>() {
            unsafe {
                Some(
                    &mut *(std::ptr::from_mut::<Entity<'w, Cx>>(self) as *mut RawEntity<'w, T, Cx>),
                )
            }
        } else {
            None
        }
    }

    /// Whether the type of data in this entity can be safely downcast
    /// into the target type.
    #[inline]
    pub fn matches_type<T>(&self) -> bool {
        self.data.type_id() == typeid::of::<T>()
    }
}
