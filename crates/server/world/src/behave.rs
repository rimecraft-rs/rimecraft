//! Server-only behaviors, corresponding to the `behave` module in `rimecraft-world`.

use block::BlockState;
use local_cx::ProvideLocalCxTy;
use voxel_math::BlockPos;
use world::{ArcAccess, World, chunk::ChunkCx};

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
