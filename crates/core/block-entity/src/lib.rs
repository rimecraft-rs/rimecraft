//! Rimecraft block entity primitives.

use std::{any::TypeId, fmt::Debug};

use erased_serde::{serialize_trait_object, Serialize as ErasedSerialize};

use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::{entry::RefEntry, Reg};
use rimecraft_serde_update::{erased::ErasedUpdate, update_trait_object};
use rimecraft_voxel_math::BlockPos;
use serde::{Deserializer, Serialize};

pub use rimecraft_downcast::ToStatic;

pub mod deser_nbt;

/// A type of [`BlockEntity`].
pub trait RawBlockEntityType<Cx>: Debug
where
    Cx: ProvideBlockStateExtTy,
{
    /// Whether the block entity supports the given state.
    fn supports(&self, state: &BlockState<'_, Cx>) -> bool;

    /// Creates a new instance of the block entity.
    fn instantiate<'w>(
        &self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
    ) -> Option<Box<BlockEntity<'w, Cx>>>;
}

/// A type of [`BlockEntity`] that can be used in a type erased context.
pub type RawBlockEntityTypeDyn<'r, Cx> = Box<dyn RawBlockEntityType<Cx> + Send + Sync + 'r>;

/// A type of [`BlockEntity`].
pub type BlockEntityType<'r, Cx> = Reg<'r, <Cx as ProvideIdTy>::Id, RawBlockEntityTypeDyn<'r, Cx>>;

/// An object holding extra data about a block in a world.
pub struct RawBlockEntity<'a, T: ?Sized, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    ty: BlockEntityType<'a, Cx>,
    pos: BlockPos,
    removed: bool,
    cached_state: BlockState<'a, Cx>,

    data: T,
}

impl<'a, T, Cx> RawBlockEntity<'a, T, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Creates a new block entity.
    pub fn new(
        ty: BlockEntityType<'a, Cx>,
        pos: BlockPos,
        state: BlockState<'a, Cx>,
        data: T,
    ) -> Self {
        Self {
            ty,
            pos,
            removed: false,
            cached_state: state,
            data,
        }
    }
}

impl<T: ?Sized, Cx> RawBlockEntity<'_, T, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Gets the immutable inner data of this block entity.
    #[inline]
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Gets the mutable inner data of this block entity.
    #[inline]
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Gets the position of this block entity.
    #[inline]
    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    /// Marks this block entity as not removed.
    #[inline]
    pub fn cancel_removal(&mut self) {
        self.removed = false;
    }

    /// Marks this block entity as removed.
    #[inline]
    pub fn mark_removed(&mut self) {
        self.removed = true;
    }

    /// Whether this block entity is marked as removed.
    #[inline]
    pub fn is_removed(&self) -> bool {
        self.removed
    }
}

impl<T, Cx> Debug for RawBlockEntity<'_, T, Cx>
where
    Cx: ProvideBlockStateExtTy + Debug,
    Cx::BlockStateExt: Debug,
    Cx::Id: Debug,
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockEntity")
            .field("type", &<&RefEntry<_, _>>::from(self.ty).key().value())
            .field("pos", &self.pos)
            .field("removed", &self.removed)
            .field("cached_state", &self.cached_state)
            .field("data", &&self.data)
            .finish()
    }
}

/// Type erased block entity data.
pub trait ErasedData
where
    Self: ErasedSerialize + for<'de> ErasedUpdate<'de> + Send + Sync + Debug + sealed::Sealed,
{
    /// The [`TypeId`] of this data.
    fn type_id(&self) -> TypeId;
}

serialize_trait_object! { ErasedData }
update_trait_object! { ErasedData }

mod sealed {
    pub trait Sealed {}
}

impl<T> sealed::Sealed for T where T: ErasedSerialize + for<'de> ErasedUpdate<'de> + Send + Sync {}

impl<T> ErasedData for T
where
    T: ErasedSerialize + for<'de> ErasedUpdate<'de> + ToStatic + Debug + Send + Sync,
{
    #[inline]
    fn type_id(&self) -> TypeId {
        TypeId::of::<<Self as ToStatic>::StaticRepr>()
    }
}

/// A type-erased variant of [`RawBlockEntity`].
pub type BlockEntity<'w, Cx> = RawBlockEntity<'w, dyn ErasedData + 'w, Cx>;

impl<'w, Cx> BlockEntity<'w, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Downcasts this type erased block entity into block entity with a concrete data type.
    ///
    /// This function returns an immutable reference if the type matches.
    pub fn downcast_ref<T: ToStatic>(&self) -> Option<&RawBlockEntity<'w, T, Cx>> {
        if self.matches_type::<T>() {
            unsafe {
                Some(&*(self as *const BlockEntity<'w, Cx> as *const RawBlockEntity<'w, T, Cx>))
            }
        } else {
            None
        }
    }

    /// Downcasts this type erased block entity into block entity with a concrete data type.
    ///
    /// This function returns a mutable reference if the type matches.
    pub fn downcast_mut<T: ToStatic>(&mut self) -> Option<&mut RawBlockEntity<'w, T, Cx>> {
        if self.matches_type::<T>() {
            unsafe {
                Some(&mut *(self as *mut BlockEntity<'w, Cx> as *mut RawBlockEntity<'w, T, Cx>))
            }
        } else {
            None
        }
    }

    /// Whether the type of data in this block entity can be safely downcasted
    /// into the target type.
    #[inline]
    pub fn matches_type<T: ToStatic>(&self) -> bool {
        self.data.type_id() == TypeId::of::<T::StaticRepr>()
    }
}

impl<T, Cx> Serialize for RawBlockEntity<'_, T, Cx>
where
    Cx: ProvideBlockStateExtTy,
    T: ?Sized + Serialize,
    Cx::Id: Serialize,
{
    /// Serializes this block entity's data and type id.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("id", &<&RefEntry<_, _>>::from(self.ty))?;
        self.data
            .serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl<'de, T, Cx> rimecraft_serde_update::Update<'de> for RawBlockEntity<'_, T, Cx>
where
    Cx: ProvideBlockStateExtTy,
    T: rimecraft_serde_update::Update<'de> + ?Sized,
{
    #[inline]
    fn update<D>(&mut self, deserializer: D) -> Result<(), <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        self.data.update(deserializer)
    }
}

/// A trait for providing block entities.
///
/// This should be implemented for [`ProvideBlockStateExtTy::BlockStateExt`]s.
pub trait ProvideBlockEntity<'w, Cx>
where
    Cx: ProvideBlockStateExtTy<BlockStateExt = Self>,
{
    /// Whether this block has a block entity.
    #[inline]
    fn has_block_entity(&self) -> bool {
        self.block_entity_constructor().is_some()
    }

    /// Gets the block entity constructor of this block.
    fn block_entity_constructor<'s>(
        &'s self,
    ) -> Option<impl FnOnce(BlockPos) -> Box<BlockEntity<'w, Cx>> + 's>;
}
