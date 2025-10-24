use std::ops::DerefMut as _;

use block::BlockState;
use block_entity::BlockEntityCell;
use fluid::BsToFs;
use local_cx::{LocalContext, dsyn_instanceof, dsyn_ty};
use parking_lot::MutexGuard;
use serde::de::DeserializeOwned;
use voxel_math::BlockPos;
use world::{
    behave::BlockEntityConstructor,
    chunk::{ChunkCx, ComputeIndex, CreationType, WorldChunk, WorldChunkLocalCx},
    event::ServerChunkEventCallback,
    view::block::SetBlockStateFlags,
};

use crate::{behave::*, chunk::ServerWorldChunkAccess};

/// Built-in callback when a block state is been replaced inside a chunk.
pub fn builtin_callback_replace_block_state<'w, Cx, Chunk>(
    pos: BlockPos,
    new: BlockState<'w, Cx>,
    old: BlockState<'w, Cx>,
    flags: SetBlockStateFlags,
    chunk: &mut Chunk,
) where
    Cx: ChunkCx<'w>
        + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>
        + BsToFs<'w>
        + ServerChunkEventCallback<'w, Chunk>,
    Cx::LocalContext<'w>: LocalContext<dsyn::Type<BlockAlwaysReplaceState>>
        + LocalContext<dsyn::Type<BlockOnStateReplaced<Cx>>>
        + LocalContext<dsyn::Type<BlockEntityOnBlockReplaced<Cx>>>
        + WorldChunkLocalCx<'w, Cx>,
    Cx::Id: DeserializeOwned,
    Chunk: ServerWorldChunkAccess<'w, Cx>,
{
    let block_diff = old.block != new.block;
    let local_cx = chunk.local_context();
    if !flags.contains(SetBlockStateFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK)
        && block_diff
        && dsyn_instanceof!(local_cx, &*old.block => BlockEntityConstructor<Cx>)
    {
        WorldChunk::__peek_block_entity_typed(
            //SAFETY: lifetime does not matter here
            unsafe { chunk.reclaim_unsafe() },
            pos,
            |cell| {
                let mut guard = cell.lock();
                let f = (**guard.ty())
                    .descriptors()
                    .get(dsyn_ty!(local_cx => BlockEntityOnBlockReplaced<Cx>))
                    .unwrap_or(default_block_entity_on_block_replaced());
                f(
                    &mut **guard,
                    chunk.wca_as_wc().world_ptr(),
                    pos,
                    old,
                    local_cx,
                    BlockEntityOnBlockReplacedMarker,
                );
            },
            CreationType::Immediate,
        );
    }

    if (block_diff || {
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

/// Built-in callback when a block state is added inside a chunk.
pub fn builtin_callback_add_block_state<'w, Cx, Chunk>(
    pos: BlockPos,
    new: BlockState<'w, Cx>,
    old: BlockState<'w, Cx>,
    flags: SetBlockStateFlags,
    chunk: &mut Chunk,
) where
    Cx: ChunkCx<'w>,
    Chunk: ServerWorldChunkAccess<'w, Cx>,
    Cx::LocalContext<'w>: LocalContext<dsyn::Type<BlockOnBlockAdded<Cx>>>,
{
    if !flags.contains(SetBlockStateFlags::SKIP_BlOCK_ADDED_CALLBACK) {
        let local_cx = chunk.local_context();
        let f = dsyn_instanceof!(local_cx, &*new.block => export BlockOnBlockAdded<Cx>)
            .unwrap_or(default_block_on_block_added());
        f(
            new,
            old,
            chunk.wca_as_wc().world_ptr(),
            pos,
            flags.contains(SetBlockStateFlags::MOVED),
            local_cx,
            BlockOnBlockAddedMarker,
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
