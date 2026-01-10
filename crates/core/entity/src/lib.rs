//! Rimecraft Entity primitives.

use std::{
    any::TypeId,
    fmt::Debug,
    sync::{
        Arc,
        atomic::{self, AtomicU32},
    },
};

use block::{BlockState, ProvideBlockStateExtTy};
use dsyn::HoldDescriptors;
use erased_serde::Serialize as ErasedSerialize;
use glam::DVec3;
use global_cx::{
    GlobalContext, ProvideIdTy, ProvideNbtTy,
    rand::{LockedRng as _, ProvideRng, Rng as _},
};
use local_cx::{PeekLocalContext, ProvideLocalCxTy};
use parking_lot::Mutex;
use rcutil::{Invariant, InvariantLifetime, PhantomInvariant, phantom_invariant};
use registry::Reg;
use serde::Serialize;
use serde_update::erased::ErasedUpdate;
use uuid::Uuid;
use voxel_math::{BlockPos, ChunkPos};

use crate::data::{DataTracked, DataTracker, DataTrackerBuilder, EntityDataCx, SerializedEntry};

mod _serde;
mod filter;
mod hit;

pub mod data;

pub use filter::*;
pub use hit::*;

/// Global context types satisfying use of entities.
pub trait EntityCx<'a>:
    ProvideIdTy
    + ProvideNbtTy
    + ProvideEntityExtTy
    + ProvideBlockStateExtTy
    + ProvideLocalCxTy
    + EntityDataCx<'a>
{
    /// The data type of a player entity, can either be a concrete type or a supertype.
    type PlayerEntityData: ?Sized + 'a;

    /// The maximum position of an entity in X and Z axis.
    const POS_XZ_BOUND: f64 = 3.0000512e7;
    /// The maximum position of an entity in Y axis.
    const POS_Y_BOUND: f64 = 2.0e7;
    /// The maximum velocity of an entity.
    const VELOCITY_BOUND: f64 = 10.0;
}

/// Global context types providing an extension types to entities.
pub trait ProvideEntityExtTy: GlobalContext {
    /// The extension type to entities, which will be treated as inlined fields when
    /// serializing and deserializing.
    ///
    /// Fields like `fall_distance`, `Glowing` in vanilla Minecraft should be contained
    /// in here.
    type EntityExt<'a>: Serialize + for<'de> serde_update::Update<'de>;
}

/// A message sent to the entity manager.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ChangeMessage {
    /// The entity being changed.
    pub entity: *const (),
    /// The variant of the change message.
    pub variant: ChangeMessageVariant,
    /// The chunk section position of the entity.
    pub section_pos: u64,
}

unsafe impl Send for ChangeMessage {}
unsafe impl Sync for ChangeMessage {}

/// A message variant for an [`ChangeMessage`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ChangeMessageVariant {
    /// The entity's position is updated.
    UpdatePos,
    /// The entity is removed.
    Remove(&'static RemovalReason),
}

/// A listener for entity changes.
///
/// The callback is simulated by a channel sending [`ChangeMessage`]s.
#[derive(Debug)]
pub struct ChangeListener {
    sender: crossbeam_channel::Sender<ChangeMessage>,
    section_pos: u64,
}

impl ChangeListener {
    /// Creates a new change listener.
    #[inline]
    pub fn new(sender: crossbeam_channel::Sender<ChangeMessage>, section_pos: u64) -> Self {
        Self {
            sender,
            section_pos,
        }
    }
}

/// Boxed entity cell with internal mutability and reference-counting.
pub type EntityCell<'w, Cx, T = dyn ErasedData<'w, Cx> + 'w> =
    Arc<Mutex<Box<RawEntity<'w, T, Cx>>>>;

/// A type of [`Entity`].
pub trait RawEntityType<'a, Cx>: HoldDescriptors<'static, 'a>
where
    Cx: EntityCx<'a>,
{
    /// Data type of the target entity.
    type Data: Data<'a, Cx, EntityType = Self>;

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
    fn erased_typeid(&self) -> TypeId;
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

    #[inline]
    fn erased_typeid(&self) -> TypeId {
        typeid::of::<T>()
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
    net_id: u32,
    ty: EntityType<'a, Cx>,
    uuid: Uuid,

    data_tracker: DataTracker<'a, Cx>,

    pos: DVec3,
    vehicle: Option<EntityCell<'a, Cx>>,
    velocity: DVec3,
    yaw: f32,
    pitch: f32,
    removal: Option<&'static RemovalReason>,
    passengers: Box<[EntityCell<'a, Cx>]>,

    /// Whether the updated velocity needs to be handled.
    pub velocity_dirty: bool,

    block_pos: BlockPos,
    chunk_pos: ChunkPos,
    bs_at_pos: Option<BlockState<'a, Cx>>,

    last_pos: DVec3,
    last_yaw: f32,
    last_pitch: f32,

    change_listener: Option<ChangeListener>,

    custom_compound: Cx::Compound,
    ext: Cx::EntityExt<'a>,

    _ghost: PhantomInvariant<'a>,

    data: T,
}

// SAFETY: guaranteed by `_ghost`.
unsafe impl<'a, T: ?Sized, Cx> Invariant for RawEntity<'a, T, Cx>
where
    Cx: EntityCx<'a>,
{
    type Lifetime = InvariantLifetime<'a>;
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
pub trait Data<'a, Cx>: DataTracked<'a, Cx>
where
    Cx: EntityCx<'a>,
{
    /// The type of the entity.
    ///
    /// Used to ensure the uniqueness of entity-data pair.
    type EntityType: RawEntityType<'a, Cx, Data = Self>;

    /// Sets the multi-parted yaw of the entity, usually consists of head and body yaws.
    #[inline]
    fn set_yaws(&mut self, yaw: f32) {
        let _ = yaw;
    }

    /// Initializes the data tracker.
    fn init_data_tracker(builder: &mut DataTrackerBuilder<'a, Cx>);
}

/// The reason for removing an entity.
///
/// In practice this should exist as a static reference and be compared using [`std::ptr::eq`].
#[derive(Debug)]
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
#[allow(missing_docs)]
pub trait ErasedData<'a, Cx>
where
    Self: DataTracked<'a, Cx>
        + ErasedSerialize
        + for<'de> ErasedUpdate<'de>
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

    fn erased_set_yaws(&mut self, yaw: f32);
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
    #[inline]
    fn erased_set_yaws(&mut self, yaw: f32) {
        self.set_yaws(yaw);
    }
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

/// Counter of entity networking ID which should be provided by the local context.
#[derive(Debug)]
pub struct IdCounter {
    inner: AtomicU32,
}

impl<'w, T, Cx> RawEntity<'w, T, Cx>
where
    Cx: EntityCx<'w, EntityExt<'w>: Default> + ProvideRng,
    T: Data<'w, Cx>,
{
    /// Creates a new entity.
    pub fn new<L>(ty: EntityType<'w, Cx>, data: T, cx: L) -> Self
    where
        L: PeekLocalContext<IdCounter>,
    {
        let mut tracker_builder = DataTrackerBuilder::new(ty);
        T::init_data_tracker(&mut tracker_builder);

        Self {
            net_id: cx.peek_acquire(|c| c.inner.fetch_add(1, atomic::Ordering::Relaxed)),
            ty,
            uuid: {
                let mut bytes = [0u8; 16];
                Cx::crypto_rng().lock().fill(&mut bytes);
                uuid::Builder::from_random_bytes(bytes).into_uuid()
            },
            data_tracker: tracker_builder.build(),
            pos: DVec3::ZERO,
            vehicle: None,
            velocity: DVec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            removal: None,
            passengers: Box::new([]),
            velocity_dirty: false,
            block_pos: BlockPos::ORIGIN,
            chunk_pos: ChunkPos::ORIGIN,
            bs_at_pos: None,
            last_pos: DVec3::ZERO,
            last_yaw: 0.0,
            last_pitch: 0.0,
            change_listener: None,
            custom_compound: Default::default(),
            ext: Default::default(),
            _ghost: phantom_invariant(),
            data,
        }
    }
}

impl<'w, T: ?Sized, Cx> RawEntity<'w, T, Cx>
where
    Cx: EntityCx<'w>,
{
    /// Whether this entity has a vehicle.
    #[inline]
    pub fn has_vehicle(&self) -> bool {
        self.vehicle.is_some()
    }

    /// Returns the vehicle of this entity.
    #[inline]
    pub fn vehicle(&self) -> Option<&EntityCell<'w, Cx>> {
        self.vehicle.as_ref()
    }

    /// Whether this entity has passengers.
    #[inline]
    pub fn has_passengers(&self) -> bool {
        !self.passengers.is_empty()
    }

    /// Returns the passengers of this entity.
    #[inline]
    pub fn passengers(&self) -> &[EntityCell<'w, Cx>] {
        &self.passengers
    }

    /// Returns the position of this entity.
    #[inline]
    pub fn pos(&self) -> DVec3 {
        self.pos
    }

    /// Sets the [`ChangeListener`] of this entity.
    #[inline]
    pub fn set_change_listener(&mut self, sender: Option<ChangeListener>) {
        self.change_listener = sender;
    }

    /// Sets the position of this entity.
    pub fn set_pos(&mut self, pos: DVec3) {
        self.pos = pos;
        let block_pos = pos.floor().as_ivec3().into();
        if self.block_pos != block_pos {
            self.block_pos = block_pos;
            self.chunk_pos = block_pos.into();
            self.bs_at_pos = None;
        }

        if let Some(ref listener) = self.change_listener {
            let _ = listener.sender.try_send(ChangeMessage {
                entity: std::ptr::from_ref(&*self).cast(),
                variant: ChangeMessageVariant::UpdatePos,
                section_pos: listener.section_pos,
            });
        }
    }

    /// Returns the velocity of this entity.
    #[inline]
    pub fn velocity(&self) -> DVec3 {
        self.velocity
    }

    /// Sets the velocity of this entity.
    #[inline]
    pub fn set_velocity(&mut self, velocity: DVec3) {
        self.velocity = velocity;
    }

    /// Returns the yaw of this entity.
    #[inline]
    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    /// Returns the pitch of this entity.
    #[inline]
    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    /// Sets the yaw of this entity.
    #[inline]
    pub fn set_yaw(&mut self, yaw: f32) {
        debug_assert!(yaw.is_finite(), "entity rotation should be finite");
        self.yaw = yaw;
    }

    /// Sets the pitch of this entity.
    #[inline]
    pub fn set_pitch(&mut self, pitch: f32) {
        debug_assert!(pitch.is_finite(), "entity rotation should be finite");
        self.pitch = (pitch % 360.0).clamp(-90.0, 90.0);
    }

    /// Updates the last position of this entity by setting it to its current position.
    #[inline]
    pub fn update_last_pos(&mut self) {
        self.last_pos = self.pos;
    }

    /// Updates the last rotation(yaw, pitch) of this entity by setting it to its current rotation.
    #[inline]
    pub fn update_last_rotation(&mut self) {
        self.last_yaw = self.yaw;
        self.last_pitch = self.pitch;
    }

    /// Gets the inner data of this entity.
    #[inline]
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Gets the mutable inner data of this entity.
    #[inline]
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Gets the extra data of this entity required by global context.
    #[inline]
    pub fn ext(&self) -> &Cx::EntityExt<'w> {
        &self.ext
    }

    /// Gets the mutable extra data of this entity required by global context.
    #[inline]
    pub fn ext_mut(&mut self) -> &mut Cx::EntityExt<'w> {
        &mut self.ext
    }

    /// Gets the custom compound of this entity.
    #[inline]
    pub fn custom_compound(&self) -> &Cx::Compound {
        &self.custom_compound
    }

    /// Gets the mutable custom compound of this entity.
    #[inline]
    pub fn custom_compound_mut(&mut self) -> &mut Cx::Compound {
        &mut self.custom_compound
    }

    /// Gets the network ID of this entity.
    #[inline]
    pub fn net_id(&self) -> u32 {
        self.net_id
    }

    /// Sets the network ID of this entity.
    ///
    /// This should only be done at client side.
    #[inline]
    pub fn set_net_id(&mut self, net_id: u32) {
        self.net_id = net_id;
    }

    /// Returns the type of this entity.
    #[inline]
    pub fn entity_type(&self) -> EntityType<'w, Cx> {
        self.ty
    }

    /// Returns the data tracker of this entity.
    #[inline]
    pub fn data_tracker(&self) -> &DataTracker<'w, Cx> {
        &self.data_tracker
    }

    /// Returns the mutable data tracker of this entity.
    #[inline]
    pub fn data_tracker_mut(&mut self) -> &mut DataTracker<'w, Cx> {
        &mut self.data_tracker
    }
}

impl<'w, T: ?Sized, Cx> RawEntity<'w, T, Cx>
where
    Cx: EntityCx<'w>,
    T: ErasedData<'w, Cx>,
    Cx::EntityExt<'w>: DataTracked<'w, Cx>,
{
    /// Updates the tracker entries.
    ///
    /// This function performs [`DataTracker::update_entries`] internally, while updating data
    /// nested inside this entity.
    pub fn update_tracker_entries<'borrow, I, F>(&mut self, entries: I)
    where
        I: IntoIterator<Item = SerializedEntry<'borrow, 'w, Cx>>,
        'w: 'borrow,
    {
        self.data_tracker.update_entries(entries, |td| {
            self.ext.on_tracked_data_set(&td);
            self.data.on_tracked_data_set(&td);
        });
    }
}
