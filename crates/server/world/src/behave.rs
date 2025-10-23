//! Server-only behaviors, corresponding to the `behave` module in `rimecraft-world`.

use std::sync::Arc;

use block::BlockState;
use block_entity::{BlockEntity, BlockEntityCell};
use dsyn::primitives::PointerLikeBool;
use local_cx::ProvideLocalCxTy;
use maybe::Maybe;
use voxel_math::BlockPos;
use world::{ArcAccess, World, chunk::ChunkCx};

use crate::game_event::DynListener;

/// Marker type for [`BlockOnStateReplaced`] to make it differs from other functions.
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockOnStateReplacedMarker;

/// Behavior of a block when its block was been replaced by another one on server side.
///
/// See [`BlockAlwaysReplaceState`] for always triggering this behavior.
///
/// # Parameters
///
/// 1. The old block state (this block).
/// 2. The server world.
/// 3. The block position.
/// 4. Whether the block was **moved** by thing like piston.
pub type BlockOnStateReplaced<Cx> = for<'env> fn(
    BlockState<'env, Cx>,
    &(dyn ArcAccess<World<'env, Cx>> + '_),
    BlockPos,
    bool,
    <Cx as ProvideLocalCxTy>::LocalContext<'env>,
    BlockOnStateReplacedMarker,
);

/// The default implementation of [`BlockOnStateReplaced`], which does nothing.
#[inline]
pub const fn default_block_on_state_replaced<'w, Cx>() -> BlockOnStateReplaced<Cx>
where
    Cx: ChunkCx<'w>,
{
    |_, _, _, _, _, _| {}
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
pub struct BlockAlwaysReplaceState(PointerLikeBool);

/// The default implementation of [`BlockAlwaysReplaceState`], which is zero.
#[inline]
pub const fn default_block_always_replace_state() -> BlockAlwaysReplaceState {
    BlockAlwaysReplaceState(PointerLikeBool::new(false))
}

impl From<BlockAlwaysReplaceState> for bool {
    #[inline]
    fn from(value: BlockAlwaysReplaceState) -> Self {
        value.0.into()
    }
}

impl From<bool> for BlockAlwaysReplaceState {
    #[inline]
    fn from(value: bool) -> Self {
        BlockAlwaysReplaceState(PointerLikeBool::new(value))
    }
}
