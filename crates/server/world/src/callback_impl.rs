use std::ops::DerefMut as _;

use block::BlockState;
use block_entity::BlockEntityCell;
use local_cx::{LocalContext, dsyn_instanceof};
use parking_lot::MutexGuard;
use voxel_math::BlockPos;
use world::{chunk::ChunkCx, view::block::SetBlockStateFlags};

use crate::{behave::*, chunk::ServerWorldChunkAccess};

/// Built-in callback when a block state is replaced inside a chunk.
pub fn builtin_callback_replace_block_state<'w, Cx, Chunk>(
    pos: BlockPos,
    new: BlockState<'w, Cx>,
    old: BlockState<'w, Cx>,
    flags: SetBlockStateFlags,
    chunk: &mut Chunk,
) where
    Cx: ChunkCx<'w>,
    Cx::LocalContext<'w>: LocalContext<dsyn::Type<BlockAlwaysReplaceState>>
        + LocalContext<dsyn::Type<BlockOnStateReplaced<Cx>>>,
    Chunk: ServerWorldChunkAccess<'w, Cx>,
{
    let local_cx = chunk.local_cx();
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
            chunk.wca_as_wc().world_ptr(),
            pos,
            flags.contains(SetBlockStateFlags::MOVED),
            local_cx,
            BlockOnStateReplacedMarker,
        );
    }
}

/// Built-in callback when a block entity is added to a chunk.
pub fn builtin_callback_add_block_entity<'w, Cx, Chunk>(
    be: &BlockEntityCell<'w, Cx>,
    chunk: &mut Chunk,
) where
    Cx: ChunkCx<'w>,
    Chunk: ServerWorldChunkAccess<'w, Cx>,
    Cx::LocalContext<'w>: LocalContext<dsyn::Type<BlockEntityGetGameEventListener<Cx>>>,
{
    crate::chunk::wc_update_game_event_listener(
        chunk.reclaim_server(),
        be,
        MutexGuard::map(be.lock(), Box::deref_mut),
    );
}

/// Built-in callback when a block entity is removed from a chunk.
pub fn builtin_callback_remove_block_entity<'w, Cx, Chunk>(
    be: &BlockEntityCell<'w, Cx>,
    chunk: &mut Chunk,
) where
    Cx: ChunkCx<'w>,
    Chunk: ServerWorldChunkAccess<'w, Cx>,
    Cx::LocalContext<'w>: LocalContext<dsyn::Type<BlockEntityGetGameEventListener<Cx>>>,
{
    crate::chunk::wc_remove_game_event_listener(
        chunk.reclaim_server(),
        be,
        MutexGuard::map(be.lock(), Box::deref_mut),
    );
}
