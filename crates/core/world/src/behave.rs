//! Traits for representing behaviors of in-game components.
//!
//! Types listed there are usually used as descriptor types.

use local_cx::ProvideLocalCxTy;
use rimecraft_block::BlockState;
use rimecraft_block_entity::BlockEntity;
use rimecraft_voxel_math::BlockPos;

use crate::{ArcAccess, ServerWorld, World, chunk::ChunkCx};

pub use rimecraft_block_entity::BlockEntityConstructorMarker;

/// Constructor of a block entity.
///
/// # Parameters
///
/// 1. Position of the block entity to construct.
/// 2. State of the block the block entity will be placed.
/// 3. Local context.
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
/// 5. Local context.
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

/// Marker type for [`BlockOnStateReplaced`] to make it differs from other functions.
#[derive(Debug, Clone, Copy, Default)]
pub struct BlockOnStateReplacedMarker;

/// Behavior of a block when its block was been replaced.
/// See [`BlockAlwaysReplaceState`] for always triggering this behavior.
///
/// # Parameters
///
/// 1. The old block state.
/// 2. The server world.
/// 3. The block position.
/// 4. Whether the block was **moved** by thing like piston.
/// 5. Local context.
pub type BlockOnStateReplaced<Cx> = for<'env> fn(
    BlockState<'env, Cx>,
    &(dyn ArcAccess<ServerWorld<'env, Cx>> + '_),
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
