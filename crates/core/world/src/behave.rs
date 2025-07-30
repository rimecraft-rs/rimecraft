//! Traits for representing behaviors of in-game components.
//!
//! Types listed there are usually used as descriptor types.

use local_cx::ProvideLocalCxTy;
use rimecraft_block::BlockState;
use rimecraft_block_entity::BlockEntity;
use rimecraft_voxel_math::BlockPos;

use crate::{ArcAccess, World, chunk::ChunkCx};

/// Constructor of a block entity.
///
/// # Parameters
///
/// 1. Position of the block entity to construct.
/// 2. State of the block the block entity will be placed.
/// 3. Local context.
pub type BlockEntityConstructor<Cx> = rimecraft_block_entity::BlockEntityConstructor<Cx>;

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
);

/// The default implementation of [`BlockEntityOnBlockReplaced`], which is an empty function.
#[inline(always)]
pub const fn default_block_entity_on_block_replaced<'w, Cx>() -> BlockEntityOnBlockReplaced<Cx>
where
    Cx: ChunkCx<'w>,
{
    |be, w, pos, state, cx| {
        let _ = (be, w, pos, state, cx);
    }
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
