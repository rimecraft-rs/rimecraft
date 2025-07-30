//! Traits for representing behaviors of in-game components.
//!
//! Types listed there are usually used as descriptor types.

use local_cx::ProvideLocalCxTy;
use rimecraft_block::BlockState;
use rimecraft_block_entity::BlockEntity;
use rimecraft_voxel_math::BlockPos;

use crate::World;

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
    &World<'env, Cx>,
    BlockPos,
    BlockState<'env, Cx>,
    <Cx as ProvideLocalCxTy>::LocalContext<'env>,
);
