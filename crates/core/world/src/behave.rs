//! Traits for representing behaviors of in-game components.
//!
//! Types listed there are usually used as descriptor types.

use local_cx::ProvideLocalCxTy;
use rimecraft_block::BlockState;
use rimecraft_voxel_math::BlockPos;

use crate::{ArcAccess, World, chunk::ChunkCx};

pub use rimecraft_block_entity::BlockEntityConstructorMarker;

/// Constructor of a block entity.
///
/// # Parameters
///
/// 1. Position of the block entity to construct.
/// 2. State of the block the block entity will be placed.
pub type BlockEntityConstructor<Cx> = rimecraft_block_entity::BlockEntityConstructor<Cx>;

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
