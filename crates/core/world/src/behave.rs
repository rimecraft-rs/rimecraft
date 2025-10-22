//! Traits for representing behaviors of in-game components.
//!
//! Types listed there are usually used as descriptor types.

use std::sync::Arc;

use local_cx::ProvideLocalCxTy;
use maybe::Maybe;
use rimecraft_block::BlockState;
use rimecraft_block_entity::{BlockEntity, BlockEntityCell};
use rimecraft_voxel_math::BlockPos;

use crate::{ArcAccess, World, chunk::ChunkCx, event::game_event::DynListener};

pub use rimecraft_block_entity::BlockEntityConstructorMarker;

/// Constructor of a block entity.
///
/// # Parameters
///
/// 1. Position of the block entity to construct.
/// 2. State of the block the block entity will be placed.
pub type BlockEntityConstructor<Cx> = rimecraft_block_entity::BlockEntityConstructor<Cx>;

/// Marker type for [`BlockEntityOnBlockReplaced`] to make it differs from other functions.
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockEntityOnBlockReplacedMarker;

/// Behavior of a block entity when its block was replaced.
///
/// # Parameters
///
/// 1. The block entity itself.
/// 2. The world block entity lies in.
/// 3. The block entity (or the block)'s position.
/// 4. The old block state.
pub type BlockEntityOnBlockReplaced<Cx> = for<'env> fn(
    &mut BlockEntity<'env, Cx>,
    &(dyn ArcAccess<World<'env, Cx>> + '_),
    BlockPos,
    BlockState<'env, Cx>,
    <Cx as ProvideLocalCxTy>::LocalContext<'env>,
    BlockEntityOnBlockReplacedMarker,
);

/// The default implementation of [`BlockEntityOnBlockReplaced`], which is an empty function.
#[inline(always)]
pub const fn default_block_entity_on_block_replaced<'w, Cx>() -> BlockEntityOnBlockReplaced<Cx>
where
    Cx: ChunkCx<'w>,
{
    |_, _, _, _, _, _| {}
}

/// Whether call block state replacement callback to the block even if the block is the same.
///
/// This is a `usize` value where zero means don't call if block unchanged, and non-zero means always call.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockAlwaysReplaceState(pub usize);

/// The default implementation of [`BlockAlwaysReplaceState`], which is zero.
#[inline]
pub const fn default_block_always_replace_state() -> BlockAlwaysReplaceState {
    BlockAlwaysReplaceState(false as usize)
}

/// Marker type for [`BlockOnBlockAdded`] to make it differs from other functions.
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockOnBlockAddedMarker;

/// Behavior of a new block when its block state is changed on server side,
/// including block placement.
///
/// # Parameters
///
/// 1. The new block state (this block).
/// 2. The old block state.
/// 3. The world.
/// 4. The block position.
/// 5. Whether to notify clients.
pub type BlockOnBlockAdded<Cx> = for<'env> fn(
    BlockState<'env, Cx>,
    BlockState<'env, Cx>,
    &(dyn ArcAccess<World<'env, Cx>> + '_),
    BlockPos,
    bool,
    <Cx as ProvideLocalCxTy>::LocalContext<'env>,
    BlockOnBlockAddedMarker,
);

/// The default implementation of [`BlockOnBlockAdded`], which does nothing.
#[inline]
pub const fn default_block_on_block_added<'w, Cx>() -> BlockOnBlockAdded<Cx>
where
    Cx: ChunkCx<'w>,
{
    |_, _, _, _, _, _, _| {}
}

/// Marker type for [`BlockEntityGetGameEventListener`] to make it differs from other functions.
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockEntityGetGameEventListenerMarker;

/// Behavior of a block entity to returns its game event listener optionally.
///
/// # Parameters
///
/// 1. The block entity itself.
pub type BlockEntityGetGameEventListener<Cx> =
    for<'env, 'r> fn(
        &'r BlockEntityCell<'env, Cx>,
        <Cx as ProvideLocalCxTy>::LocalContext<'env>,
        BlockEntityGetGameEventListenerMarker,
    ) -> Option<Maybe<'r, Arc<DynListener<'env, Cx>>>>;

/// The default implementation of [`BlockEntityGetGameEventListener`].
pub const fn default_block_entity_get_game_event_listener<'w, Cx>()
-> BlockEntityGetGameEventListener<Cx>
where
    Cx: ChunkCx<'w>,
{
    |_, _, _| None
}

// impl area

impl From<bool> for BlockAlwaysReplaceState {
    #[inline]
    fn from(value: bool) -> Self {
        Self(value as usize)
    }
}

impl From<BlockAlwaysReplaceState> for bool {
    #[inline]
    fn from(value: BlockAlwaysReplaceState) -> Self {
        value.0 != 0
    }
}
