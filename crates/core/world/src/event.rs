//! Event triggers.

use rimecraft_block::BlockState;
use rimecraft_voxel_math::BlockPos;

use crate::{ArcAccess, World, chunk::ChunkCx, view::block::SetBlockStateFlags};

pub mod game_event;

/// Server event callbacks implemented on a global context.
///
/// This should be implemented in a pure-client environment as well but will do nothing there.
pub trait ServerEventCallback<'w>: ChunkCx<'w> {
    /// Called after a block state is replaced.
    #[inline(always)]
    fn replace_block_state(
        pos: BlockPos,
        new: BlockState<'w, Self>,
        old: BlockState<'w, Self>,
        world: &impl ArcAccess<World<'w, Self>>,
        flags: SetBlockStateFlags,
        local_cx: Self::LocalContext<'w>,
    ) {
        let _ = (pos, new, old, world, flags, local_cx);
    }
}
