//! Rimecraft block entity primitives.

use std::{any::TypeId, fmt::Debug};

use ::component::{
    ErasedComponentType, RawErasedComponentType, changes::ComponentChanges, map::ComponentMap,
};
use ahash::AHashSet;
use dsyn::HoldDescriptors;
use erased_serde::{Serialize as ErasedSerialize, serialize_trait_object};

use local_cx::ProvideLocalCxTy;
use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::Reg;
use rimecraft_serde_update::erased::ErasedUpdate;
use rimecraft_voxel_math::BlockPos;

mod components_util;
pub mod serde;

pub use components_util::ComponentsAccess;

/// Re-export of `rimecraft-component`
pub mod component {
    pub use ::component::*;
}

/// A trait for providing fundamental built-in component types.
pub trait ProvideBuiltInComponentTypes<'r>: ProvideIdTy {
    /// The type of block entity data.
    fn block_entity_data() -> ErasedComponentType<'r, Self>;
}

/// A type of [`BlockEntity`].
pub trait RawBlockEntityType<'a, Cx>: HoldDescriptors<'static, 'a>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Data type of the target block entities.
    type Data;

    /// Whether the block entity supports the given state.
    fn supports(&self, state: &BlockState<'_, Cx>) -> bool;

    /// Creates a new instance of the block entity.
    fn instantiate<'w>(
        &self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
    ) -> Option<RawBlockEntity<'w, Self::Data, Cx>>;
}

/// [`RawBlockEntityType`] with type erased.
#[allow(missing_docs)]
pub trait ErasedRawBlockEntityType<'w, Cx>: HoldDescriptors<'static, 'w> + Debug
where
    Cx: ProvideBlockStateExtTy,
{
    fn erased_supports(&self, state: &BlockState<'_, Cx>) -> bool;

    /// Creates a new instance of the block entity.
    fn erased_instantiate(
        &self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
    ) -> Option<Box<BlockEntity<'w, Cx>>>;
}

impl<'w, T, Cx> ErasedRawBlockEntityType<'w, Cx> for T
where
    T: RawBlockEntityType<'w, Cx> + Debug,
    T::Data: ErasedData<'w, Cx> + 'w,
    Cx: ProvideBlockStateExtTy,
{
    fn erased_supports(&self, state: &BlockState<'_, Cx>) -> bool {
        self.supports(state)
    }

    fn erased_instantiate(
        &self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
    ) -> Option<Box<BlockEntity<'w, Cx>>> {
        self.instantiate(pos, state).map(|be| {
            let boxed: Box<BlockEntity<'w, Cx>> = Box::new(be);
            boxed
        })
    }
}

/// A type of [`BlockEntity`] that can be used in a type erased context.
pub type DynErasedRawBlockEntityType<'r, Cx> =
    Box<dyn ErasedRawBlockEntityType<'r, Cx> + Send + Sync + 'r>;

/// A type of [`BlockEntity`].
pub type BlockEntityType<'r, Cx> =
    Reg<'r, <Cx as ProvideIdTy>::Id, DynErasedRawBlockEntityType<'r, Cx>>;

/// An object holding extra data about a block in a world.
pub struct RawBlockEntity<'a, T: ?Sized, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    ty: BlockEntityType<'a, Cx>,
    pos: BlockPos,
    removed: bool,
    cached_state: BlockState<'a, Cx>,
    components: ComponentMap<'a, Cx>,

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
            components: ComponentMap::EMPTY,
        }
    }
}

impl<'a, T: ?Sized, Cx> RawBlockEntity<'a, T, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Returns the type of this block entity.
    #[inline]
    pub fn ty(&self) -> BlockEntityType<'a, Cx> {
        self.ty
    }

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

    /// Gets the component map of this block entity.
    #[inline]
    pub fn components(&self) -> &ComponentMap<'a, Cx> {
        &self.components
    }

    /// Gets the mutable component map of this block entity.
    #[inline]
    pub fn components_mut(&mut self) -> &mut ComponentMap<'a, Cx> {
        &mut self.components
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

impl<'a, T: ?Sized, Cx> RawBlockEntity<'a, T, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideBuiltInComponentTypes<'a>,
    T: Data<'a, Cx>,
{
    /// Reads components from given pair of default and changed components.
    pub fn read_components(
        &mut self,
        default: &'a ComponentMap<'a, Cx>,
        changes: ComponentChanges<'a, '_, Cx>,
    ) {
        let mut set: AHashSet<RawErasedComponentType<'a, Cx>> = [*Cx::block_entity_data()].into();
        let mut map = ComponentMap::with_changes(default, changes);
        self.data.read_components(ComponentsAccess {
            set: &mut set,
            map: &mut map,
        });

        let Some(changes) = map.changes() else {
            unreachable!()
        };
        let (added, _) = changes
            .retain(|ty| !set.contains(&*ty))
            .into_added_removed_pair();
        self.components = added;
    }
}

impl<'a, T: ?Sized, Cx> RawBlockEntity<'a, T, Cx>
where
    Cx: ProvideBlockStateExtTy,
    T: Data<'a, Cx>,
{
    /// Creates a component map from the data and inner components
    /// of this block entity.
    pub fn create_components(&self) -> ComponentMap<'a, Cx> {
        let mut builder = ComponentMap::builder();
        builder.extend(self.components.iter());
        self.data.insert_components(&mut builder);
        builder.build()
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
            .field("type", &Reg::to_id(self.ty))
            .field("pos", &self.pos)
            .field("removed", &self.removed)
            .field("cached_state", &self.cached_state)
            .field("data", &&self.data)
            .finish()
    }
}

/// A trait for generic block entity data types.
pub trait Data<'a, Cx>
where
    Cx: ProvideIdTy,
{
    /// Reads components from the given accessor.
    #[inline]
    fn read_components(&mut self, accessor: ComponentsAccess<'_, 'a, Cx>) {
        let _ = accessor;
    }

    /// Writes components to the given builder.
    #[inline]
    fn insert_components(&self, builder: &mut component::map::Builder<'a, Cx>) {
        let _ = builder;
    }
}

/// Type erased block entity data.
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
    fn type_id(&self) -> TypeId {
        typeid::of::<Self>()
    }
}

// Emit a warning
#[allow(single_use_lifetimes)]
mod ser_dyn_obj {
    use super::*;
    serialize_trait_object!(<'a, Cx> ErasedData<'a, Cx> where Cx: ProvideIdTy);
}

impl<'de, Cx> rimecraft_serde_update::Update<'de> for dyn ErasedData<'_, Cx> + '_
where
    Cx: ProvideIdTy,
{
    rimecraft_serde_update::__internal_update_from_erased!();
}
impl<'de, Cx> rimecraft_serde_update::Update<'de> for dyn ErasedData<'_, Cx> + Send + '_
where
    Cx: ProvideIdTy,
{
    rimecraft_serde_update::__internal_update_from_erased!();
}
impl<'de, Cx> rimecraft_serde_update::Update<'de> for dyn ErasedData<'_, Cx> + Sync + '_
where
    Cx: ProvideIdTy,
{
    rimecraft_serde_update::__internal_update_from_erased!();
}
impl<'de, Cx> rimecraft_serde_update::Update<'de> for dyn ErasedData<'_, Cx> + Send + Sync + '_
where
    Cx: ProvideIdTy,
{
    rimecraft_serde_update::__internal_update_from_erased!();
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
    #[inline]
    fn type_id(&self) -> TypeId {
        typeid::of::<T>()
    }
}

/// A type-erased variant of [`RawBlockEntity`].
pub type BlockEntity<'w, Cx> = RawBlockEntity<'w, dyn ErasedData<'w, Cx> + 'w, Cx>;

impl<'w, Cx> BlockEntity<'w, Cx>
where
    Cx: ProvideBlockStateExtTy,
{
    /// Downcasts this type erased block entity into block entity with a concrete data type.
    ///
    /// This function returns an immutable reference if the type matches.
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'w`.
    pub unsafe fn downcast_ref<T>(&self) -> Option<&RawBlockEntity<'w, T, Cx>> {
        if self.matches_type::<T>() {
            unsafe {
                Some(
                    &*(std::ptr::from_ref::<BlockEntity<'w, Cx>>(self)
                        as *const RawBlockEntity<'w, T, Cx>),
                )
            }
        } else {
            None
        }
    }

    /// Downcasts this type erased block entity into block entity with a concrete data type.
    ///
    /// This function returns a mutable reference if the type matches.
    ///
    /// # Safety
    ///
    /// This function could not guarantee lifetime of type `T` is sound.
    /// The type `T`'s lifetime parameters should not overlap lifetime `'w`.
    pub unsafe fn downcast_mut<T>(&mut self) -> Option<&mut RawBlockEntity<'w, T, Cx>> {
        if self.matches_type::<T>() {
            unsafe {
                Some(
                    &mut *(std::ptr::from_mut::<BlockEntity<'w, Cx>>(self)
                        as *mut RawBlockEntity<'w, T, Cx>),
                )
            }
        } else {
            None
        }
    }

    /// Whether the type of data in this block entity can be safely downcast
    /// into the target type.
    #[inline]
    pub fn matches_type<T>(&self) -> bool {
        self.data.type_id() == typeid::of::<T>()
    }
}

/// Marker type for [`BlockEntityConstructor`] to make it differs from other functions.
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockEntityConstructorMarker;

/// Constructor of a [`BlockEntity`].
///
/// This should be used as a descriptor type.
pub type BlockEntityConstructor<Cx> = for<'env> fn(
    BlockPos,
    BlockState<'env, Cx>,
    <Cx as ProvideLocalCxTy>::LocalContext<'env>,
    BlockEntityConstructorMarker,
) -> Box<BlockEntity<'env, Cx>>;
