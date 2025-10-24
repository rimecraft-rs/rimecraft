//! Event triggers.

use rimecraft_block::BlockState;
use rimecraft_block_entity::BlockEntityCell;
use rimecraft_voxel_math::BlockPos;

use crate::{WorldCx, view::block::SetBlockStateFlags};

/// Server event callbacks implemented on a global context.
///
/// This should be implemented in a pure-client environment as well but will do nothing there.
pub trait ServerChunkEventCallback<'w, Access>: WorldCx<'w> {
    /// Called after a block state is been replaced.
    #[inline(always)]
    fn replace_block_state_callback(
        pos: BlockPos,
        new: BlockState<'w, Self>,
        old: BlockState<'w, Self>,
        flags: SetBlockStateFlags,
        chunk: &mut Access,
    ) {
        let _ = (pos, new, old, flags, chunk);
    }

    /// Called after a block state is been added.
    ///
    /// This is called after [`ServerChunkEventCallback::replace_block_state_callback`] and the removal of block entity.
    #[inline(always)]
    fn add_block_state_callback(
        pos: BlockPos,
        new: BlockState<'w, Self>,
        old: BlockState<'w, Self>,
        flags: SetBlockStateFlags,
        chunk: &mut Access,
    ) {
        let _ = (pos, new, old, flags, chunk);
    }

    /// Called after a block entity is added.
    #[inline(always)]
    fn add_block_entity_callback(be: &BlockEntityCell<'w, Self>, chunk: &mut Access) {
        let _ = (be, chunk);
    }

    /// Called after a block entity is removed, and _before it is marked as removed._
    #[inline(always)]
    fn remove_block_entity_callback(be: &BlockEntityCell<'w, Self>, chunk: &mut Access) {
        let _ = (be, chunk);
    }
}
