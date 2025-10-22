use block::BlockState;
use local_cx::{LocalContext, dsyn_instanceof};
use voxel_math::BlockPos;
use world::{ArcAccess, World, behave::*, chunk::ChunkCx, view::block::SetBlockStateFlags};

use crate::behave::*;

/// Built-in callback when a block state is replaced inside a chunk.
pub fn builtin_callback_replace_block_state<'w, Cx, WA>(
    pos: BlockPos,
    new: BlockState<'w, Cx>,
    old: BlockState<'w, Cx>,
    world: &WA,
    flags: SetBlockStateFlags,
    local_cx: Cx::LocalContext<'w>,
) where
    Cx: ChunkCx<'w>,
    WA: ArcAccess<World<'w, Cx>>,
    Cx::LocalContext<'w>: LocalContext<dsyn::Type<BlockAlwaysReplaceState>>
        + LocalContext<dsyn::Type<BlockOnStateReplaced<Cx>>>,
{
    if (old.block != new.block || {
        bool::from(
            dsyn_instanceof!(local_cx, &*new.block => export BlockAlwaysReplaceState)
                .unwrap_or(default_block_always_replace_state()),
        )
    }) && (flags.contains(SetBlockStateFlags::NOTIFY_NEIGHBORS)
        || flags.contains(SetBlockStateFlags::MOVED))
    {
        let f = dsyn_instanceof!(local_cx, &*old.block => export BlockOnStateReplaced<Cx>)
            .unwrap_or(default_block_on_state_replaced());
        f(
            old,
            world,
            pos,
            flags.contains(SetBlockStateFlags::MOVED),
            local_cx,
            BlockOnStateReplacedMarker,
        );
    }
}
