//! Rimecraft Entity primitives.

use std::{any::TypeId, fmt::Debug, sync::Arc};

use dsyn::HoldDescriptors;
use erased_serde::Serialize as ErasedSerialize;
use glam::DVec3;
use global_cx::{GlobalContext, ProvideIdTy, ProvideNbtTy};
use parking_lot::Mutex;
use registry::Reg;
use serde::{Serialize, de::DeserializeOwned};
use serde_update::erased::ErasedUpdate;
use uuid::Uuid;

mod _serde;

/// Global context types satisfying use of entities.
pub trait EntityCx<'a>: ProvideIdTy + ProvideNbtTy + ProvideEntityExtTy {}

impl<T> EntityCx<'_> for T where T: ProvideIdTy + ProvideNbtTy + ProvideEntityExtTy {}

/// Global context types providing an extension type to entities.
pub trait ProvideEntityExtTy: GlobalContext {
    /// The extension type to entities, which will be treated as inlined fields when
    /// serializing and deserializing.
    ///
    /// Fields like `fall_distance`, `Glowing` in vanilla Minecraft should be contained
    /// in here.
    type EntityExt<'a>: Serialize + DeserializeOwned;
}

/// Boxed entity cell with internal mutability and reference-counting.
pub type EntityCell<'w, Cx> = Arc<Mutex<Box<Entity<'w, Cx>>>>;

/// A type of [`Entity`].
pub trait RawEntityType<'a, Cx>: HoldDescriptors<'static, 'a>
where
    Cx: EntityCx<'a>,
{
    /// Data type of the target entity.
    type Data: Data<'a, Cx>;

    /// Creates a new entity.
    fn create(&self, this: EntityType<'a, Cx>) -> RawEntity<'a, Self::Data, Cx>;

    /// Whether this entity is saveable.
    #[inline]
    fn is_saveable(&self) -> bool {
        true
    }
}

/// [`RawEntityType`] with type erased.
#[allow(missing_docs)]
pub trait ErasedRawEntityType<'a, Cx>: HoldDescriptors<'static, 'a> + Debug
where
    Cx: EntityCx<'a>,
{
    fn erased_create(&self, this: EntityType<'a, Cx>) -> Box<Entity<'a, Cx>>;
    fn erased_is_saveable(&self) -> bool;
}

impl<'a, Cx, T> ErasedRawEntityType<'a, Cx> for T
where
    T: RawEntityType<'a, Cx> + Debug + 'a,
    Cx: EntityCx<'a>,
    T::Data: ErasedData<'a, Cx>,
{
    #[inline]
    fn erased_create(&self, this: EntityType<'a, Cx>) -> Box<Entity<'a, Cx>> {
        Box::new(self.create(this))
    }

    #[inline]
    fn erased_is_saveable(&self) -> bool {
        self.is_saveable()
    }
}

/// A type of [`Entity`] that can be used in a type erased context.
pub type DynErasedRawEntityType<'r, Cx> = Box<dyn ErasedRawEntityType<'r, Cx> + Send + Sync + 'r>;

/// A type of [`Entity`].
pub type EntityType<'r, Cx> = Reg<'r, <Cx as ProvideIdTy>::Id, DynErasedRawEntityType<'r, Cx>>;

/// An object in a world with double-precision position.
pub struct RawEntity<'a, T: ?Sized, Cx>
where
    Cx: EntityCx<'a>,
{
    ty: EntityType<'a, Cx>,
    uuid: Uuid,

    pos: DVec3,
    vehicle: Option<EntityCell<'a, Cx>>,
    velocity: DVec3,
    yaw: f32,
    pitch: f32,
    removal: Option<&'static RemovalReason>,
    passengers: Option<Box<[EntityCell<'a, Cx>]>>,

    custom_compound: Cx::Compound,
    ext: Cx::EntityExt<'a>,

    data: T,
}

impl<'a, T: ?Sized, Cx> Debug for RawEntity<'a, T, Cx>
where
    Cx: EntityCx<'a, Id: Debug, Compound: Debug, EntityExt<'a>: Debug>,
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawEntity")
            .field("ty", &self.ty)
            .field("uuid", &self.uuid)
            .field("pos", &self.pos)
            .field("velocity", &self.velocity)
            .field("yaw", &self.yaw)
            .field("pitch", &self.pitch)
            .field("custom_compound", &self.custom_compound)
            .field("ext", &self.ext)
            .field("data", &&self.data)
            .finish_non_exhaustive()
    }
}

/// A trait for generic entity data types.
pub trait Data<'a, Cx>
where
    Cx: EntityCx<'a>,
{
}

/// The reason for removing an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemovalReason {
    /// Description of the reason for removing the entity.
    pub reason: &'static str,
    /// Whether the entity should be destroyed.
    pub should_destroy: bool,
    /// Whether the entity should be saved.
    pub should_save: bool,
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
    Cx: EntityCx<'a>,
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
    serialize_trait_object!(<'a, Cx> ErasedData<'a, Cx> where Cx: EntityCx<'a>);
}

impl<'a, 'de, Cx> serde_update::Update<'de> for dyn ErasedData<'a, Cx> + '_
where
    Cx: EntityCx<'a>,
{
    serde_update::__internal_update_from_erased!();
}
impl<'a, 'de, Cx> serde_update::Update<'de> for dyn ErasedData<'a, Cx> + Send + '_
where
    Cx: EntityCx<'a>,
{
    serde_update::__internal_update_from_erased!();
}
impl<'a, 'de, Cx> serde_update::Update<'de> for dyn ErasedData<'a, Cx> + Sync + '_
where
    Cx: EntityCx<'a>,
{
    serde_update::__internal_update_from_erased!();
}
impl<'a, 'de, Cx> serde_update::Update<'de> for dyn ErasedData<'a, Cx> + Send + Sync + '_
where
    Cx: EntityCx<'a>,
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
    Cx: EntityCx<'a>,
{
}

/// A type-erased variant of [`RawEntity`].
pub type Entity<'w, Cx> = RawEntity<'w, dyn ErasedData<'w, Cx> + 'w, Cx>;

impl<'w, Cx> Entity<'w, Cx>
where
    Cx: EntityCx<'w>,
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

impl<'w, T, Cx> RawEntity<'w, T, Cx>
where
    Cx: EntityCx<'w>,
{
    /// Whether this entity has a vehicle.
    #[inline]
    pub fn has_vehicle(&self) -> bool {
        self.vehicle.is_some()
    }
}
