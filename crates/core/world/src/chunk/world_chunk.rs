//! World chunks.

use dsyn::HoldDescriptors as _;
use glam::IVec3;
use ident_hash::IHashMap;
use local_cx::{LocalContext, dsyn_instanceof, dsyn_ty, dyn_cx::AsDynamicContext};
use parking_lot::{Mutex, MutexGuard};
use rimecraft_block::BlockState;
use rimecraft_block_entity::{
    BlockEntity, DynErasedRawBlockEntityType, component::RawErasedComponentType,
};
use rimecraft_fluid::{BsToFs, FluidState};
use rimecraft_registry::Registry;
use rimecraft_voxel_math::{BlockPos, coord_section_from_block};
use serde::{Deserialize, de::DeserializeSeed};

use crate::{
    DsynCache, ServerWorld, World,
    behave::*,
    chunk::{AsBaseChunkAccess, BaseChunkAccess},
    event::game_event,
    heightmap,
    view::block::*,
};

use super::{BORDER_LEN, BaseChunk, BlockEntityCell, Chunk, ChunkCx, section::ComputeIndex};

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::{Arc, Weak},
};

/// Local context bounds alias for most [`WorldChunk`] operations.
///
/// [`AsDynamicContext`] not included so you need to bound yourself.
pub trait WorldChunkLocalCx<'w, Cx>:
    LocalContext<&'w Registry<Cx::Id, RawErasedComponentType<'w, Cx>>>
    + LocalContext<&'w Registry<Cx::Id, DynErasedRawBlockEntityType<'w, Cx>>>
    + LocalContext<dsyn::Type<BlockEntityConstructor<Cx>>>
    + LocalContext<dsyn::Type<BlockEntityOnBlockReplaced<Cx>>>
    + LocalContext<dsyn::Type<BlockAlwaysReplaceState>>
    + LocalContext<dsyn::Type<BlockOnStateReplaced<Cx>>>
    + LocalContext<dsyn::Type<BlockOnBlockAdded<Cx>>>
    + LocalContext<dsyn::Type<BlockEntityGetGameEventListener<Cx>>>
where
    Cx: ChunkCx<'w>,
{
}

impl<'w, Cx, L> WorldChunkLocalCx<'w, Cx> for L
where
    Cx: ChunkCx<'w>,
    L: LocalContext<&'w Registry<Cx::Id, RawErasedComponentType<'w, Cx>>>
        + LocalContext<&'w Registry<Cx::Id, DynErasedRawBlockEntityType<'w, Cx>>>
        + LocalContext<dsyn::Type<BlockEntityConstructor<Cx>>>
        + LocalContext<dsyn::Type<BlockEntityOnBlockReplaced<Cx>>>
        + LocalContext<dsyn::Type<BlockAlwaysReplaceState>>
        + LocalContext<dsyn::Type<BlockOnStateReplaced<Cx>>>
        + LocalContext<dsyn::Type<BlockOnBlockAdded<Cx>>>
        + LocalContext<dsyn::Type<BlockEntityGetGameEventListener<Cx>>>,
{
}

/// Chunk for worlds.
pub struct WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// The `BaseChunk`.
    pub base: BaseChunk<'w, Cx>,

    is_client: bool,
    loaded_to_world: bool,
    game_event_dispatchers: Mutex<IHashMap<i32, Arc<game_event::Dispatcher<'w, Cx>>>>,
    world_ptr: Weak<World<'w, Cx>>,

    local_cx: Cx::LocalContext<'w>,
    dsyn_cache: Arc<DsynCache<'w, Cx>>,
}

impl<'w, Cx> Debug for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + Debug,
    Cx::Id: Debug,
    Cx::BlockStateExt<'w>: Debug,
    Cx::BlockStateList: Debug,
    Cx::FluidStateExt: Debug,
    Cx::Biome: Debug,
    Cx::BiomeList: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldChunk")
            .field("base", &self.base)
            .finish_non_exhaustive()
    }
}

/// The type of `BlockEntity` creation in [`WorldChunk`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CreationType {
    /// Immediate creation.
    Immediate,
    /// Queued creation.
    Queued,
    /// Checks if the block entity exists.
    Check,
}

impl<'w, Cx> WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn can_tick_block_entities(&self) -> bool {
        self.loaded_to_world || self.is_client
    }
}

impl<'w, Cx> WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
{
    fn __peek_game_event_dispatcher<F, T>(
        this: impl WorldChunkAccess<'w, Cx>,
        y_section_coord: i32,
        f: F,
    ) -> Option<T>
    where
        F: for<'env> FnOnce(&'env Arc<game_event::Dispatcher<'w, Cx>>) -> T,
    {
        if this.wca_as_wc().is_client {
            None
        } else {
            let mut g = this.write_game_event_dispatchers();
            if let Some(d) = g.get(&y_section_coord) {
                Some(f(d))
            } else {
                let d = Arc::new(game_event::Dispatcher::new());
                let result = f(&d);
                g.insert(y_section_coord, d);
                Some(result)
            }
        }
    }
    fn __update_game_event_listener(
        mut this: impl WorldChunkAccess<'w, Cx>,
        be_cell: &BlockEntityCell<'w, Cx>,
        be: impl Deref<Target = BlockEntity<'w, Cx>>,
    ) {
        let y = be.pos().y();
        let listener_fn = dsyn_instanceof!(cached this.dsyn_cache(), this.local_cx(), &*be => export BlockEntityGetGameEventListener<Cx>)
            .unwrap_or(default_block_entity_get_game_event_listener());

        // release the guard
        drop(be);

        if let Some(listener) = listener_fn(
            be_cell,
            this.local_cx(),
            BlockEntityGetGameEventListenerMarker,
        ) {
            let _ = Self::__peek_game_event_dispatcher(
                this.reclaim(),
                coord_section_from_block(y),
                |d| {
                    d.push_erased(match listener {
                        maybe::Maybe::Borrowed(a) => a.clone(),
                        maybe::Maybe::Owned(maybe::SimpleOwned(a)) => a,
                    })
                },
            );
        }
    }
}

impl<'w, Cx> WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    fn __peek_block_entity_typed<F, T>(
        mut this: impl WorldChunkAccess<'w, Cx>,
        pos: BlockPos,
        pk: F,
        ty: CreationType,
    ) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T,
    {
        let mut bes = this.reclaim().bca().write_block_entities();
        let be = bes.get(&pos).cloned();
        if let Some(ref be) = be {
            if be.lock().is_removed() {
                bes.remove(&pos);
                return None;
            }
        } else {
            drop(bes);
            let nbt_opt = this.reclaim().bca().write_block_entity_nbts().remove(&pos);
            if let Some(nbt) = nbt_opt
                && let Some(be2) = Self::__load_block_entity(this.reclaim(), pos, nbt)
            {
                return Some(pk(&be2));
            }
            if ty == CreationType::Immediate
                && let Some(be) = Self::__create_block_entity(this.reclaim(), pos)
            {
                Self::__add_block_entity(this.reclaim(), be)
            }
        }

        be.as_ref().map(pk)
    }

    /// Peeks a [`BlockEntity`] at the target location, with given [`CreationType`].
    #[inline]
    pub fn peek_block_entity_typed<F, T>(&self, pos: BlockPos, pk: F, ty: CreationType) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T,
    {
        Self::__peek_block_entity_typed(self, pos, pk, ty)
    }

    /// Peeks a [`BlockEntity`] at the target location, with given [`CreationType`].
    #[inline]
    pub fn peek_block_entity_typed_lf<F, T>(
        &mut self,
        pos: BlockPos,
        pk: F,
        ty: CreationType,
    ) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T,
    {
        Self::__peek_block_entity_typed(self, pos, pk, ty)
    }

    fn __add_block_entity(
        mut this: impl WorldChunkAccess<'w, Cx>,
        block_entity: Box<BlockEntity<'w, Cx>>,
    ) {
        if let Some(cell) = Self::__set_block_entity(this.reclaim(), block_entity, true) {
            if !this.wca_as_wc().is_client {
                Self::__update_game_event_listener(
                    this.reclaim(),
                    &cell,
                    MutexGuard::map(cell.lock(), |g| &mut **g),
                );
            }
            //PLACEHOLDER
            let _ = ();
            //TODO: Update tickers
        }
    }

    /// Adds a block entity to this chunk.
    #[inline]
    pub fn add_block_entity(&mut self, block_entity: Box<BlockEntity<'w, Cx>>) {
        Self::__add_block_entity(self, block_entity);
    }

    /// Adds a block entity to this chunk.
    #[inline]
    pub fn add_block_entity_locked(&self, block_entity: Box<BlockEntity<'w, Cx>>) {
        Self::__add_block_entity(self, block_entity);
    }

    fn __load_block_entity(
        mut this: impl WorldChunkAccess<'w, Cx>,
        pos: BlockPos,
        nbt: Cx::Compound,
    ) -> Option<BlockEntityCell<'w, Cx>> {
        let be = DeserializeSeed::deserialize(
            rimecraft_block_entity::serde::Seed {
                pos,
                state: Self::__block_state(this.reclaim(), pos).unwrap(),
                local_cx: this.wca_as_wc().local_cx,
            },
            Cx::compound_to_deserializer(&nbt),
        )
        .ok();

        if let Some(be) = be {
            Self::__add_block_entity(this.reclaim(), be);
            Some(
                this.reclaim()
                    .bca()
                    .read_block_entities()
                    .get(&pos)
                    .expect("block entity should be inserted into this chunk")
                    .clone(),
            )
        } else {
            None
        }
    }

    fn __create_block_entity(
        mut this: impl WorldChunkAccess<'w, Cx>,
        pos: BlockPos,
    ) -> Option<Box<BlockEntity<'w, Cx>>> {
        let bs = Self::__block_state(this.reclaim(), pos)?;
        dsyn_instanceof!(cached this.dsyn_cache(), this.local_cx(), &*bs.block => export BlockEntityConstructor<Cx>)
                .map(|f| f(pos, bs, this.local_cx(), BlockEntityConstructorMarker))
    }

    fn __set_block_entity(
        mut this: impl WorldChunkAccess<'w, Cx>,
        mut block_entity: Box<BlockEntity<'w, Cx>>,
        return_cell: bool,
    ) -> Option<BlockEntityCell<'w, Cx>> {
        let dsyn_ty =
            dsyn_ty!(cached this.dsyn_cache(), this.local_cx() => BlockEntityConstructor<Cx>);
        if Self::__block_state(this.reclaim(), block_entity.pos())
            .map(|bs| (*bs.block).descriptors().contains(dsyn_ty))
            .unwrap_or_default()
        {
            block_entity.cancel_removal();
            let pos = block_entity.pos();
            let cell = Arc::new(Mutex::new(block_entity));
            let return_val = return_cell.then(|| cell.clone());
            let mut be2 = this
                .reclaim()
                .bca()
                .write_block_entities()
                .insert(pos, cell);
            if let Some(be) = &mut be2 {
                if let Some(be) = Arc::get_mut(be) {
                    be.get_mut().mark_removed();
                } else {
                    be.lock().mark_removed();
                }
            }
            return_val
        } else {
            None
        }
    }

    fn __remove_block_entity(
        mut this: impl WorldChunkAccess<'w, Cx>,
        pos: BlockPos,
    ) -> Option<BlockEntityCell<'w, Cx>> {
        if this.wca_as_wc().can_tick_block_entities() {
            let mut be = this.reclaim().bca().write_block_entities().remove(&pos);
            if let Some(be) = &mut be {
                //TODO: remove game event listener
                if let Some(raw) = Arc::get_mut(be) {
                    raw.get_mut().mark_removed();
                } else {
                    be.lock().mark_removed();
                }
            }
            //TODO: remove ticker
            be
        } else {
            None
        }
    }

    fn __set_block_state(
        mut this: impl WorldChunkAccess<'w, Cx>,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
        flags: SetBlockStateFlags,
    ) -> Option<BlockState<'w, Cx>> {
        #[cfg(feature = "tracing")]
        let _span =
            tracing::trace_span!("set block state", pos = %pos, block = %state.block).entered();

        let local_cx = this.local_cx();
        let is_client = this.wca_as_wc().is_client;

        let section_index = this.wca_as_bc().height_limit.section_index(pos.y());
        let mut section = this.reclaim().bca().write_chunk_section(section_index)?;

        // Skip setting empty block (eg air) inside empty sections.
        if section.is_empty() && state.block.settings().is_empty {
            return None;
        }

        // Set state inside chunk section.
        let pos_sec = pos.0 & (BORDER_LEN as i32 - 1);
        let old_state =
            section.set_block_state(pos_sec.x as u32, pos_sec.y as u32, pos_sec.z as u32, state);

        drop(section);

        // Aftermath

        // Skip handling identical state
        if std::ptr::eq(old_state.state, state.state) {
            return None;
        }

        // Update height maps
        // (vanilla) MOTION_BLOCKING,MOTION_BLOCKING_NO_LEAVES, OCEAN_FLOOR, OCEAN_FLOOR, WORLD_SURFACE.
        for ty in <Cx::HeightmapType as heightmap::Type<'w, Cx>>::iter_block_update_types_wc() {
            //SAFETY: This is safe because the `hms` is a valid pointer, and `__block_state` does not interact with heightmaps.
            let mut this_short_life = unsafe { this.reclaim_unsafe() };
            if let Some(hm) = this.reclaim().bca().write_heightmaps().get_mut(ty) {
                hm.track_update(pos_sec.x, pos.y(), pos_sec.z, state, |pos, pred| {
                    pred(Self::__block_state(this_short_life.reclaim(), pos))
                });
            }
        }

        //TODO: update chunk manager
        //TODO: update lighting

        // Remove old block entity if present
        if old_state.block != state.block
            && dsyn_instanceof!(cached this.dsyn_cache(), local_cx, &*old_state.block => BlockEntityConstructor<Cx>)
        {
            if !is_client
                && !flags.contains(SetBlockStateFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK)
            {
                let ty =
                    dsyn_ty!(cached this.dsyn_cache(), local_cx => BlockEntityOnBlockReplaced<Cx>);
                let wa = this.wca_as_wc().world_ptr.clone();

                Self::__peek_block_entity_typed(
                    this.reclaim(),
                    pos,
                    |cell| {
                        let mut guard = cell.lock();
                        let f = (**guard.ty())
                            .descriptors()
                            .get(ty)
                            .unwrap_or(default_block_entity_on_block_replaced());
                        f(
                            &mut **guard,
                            &wa,
                            pos,
                            old_state,
                            local_cx,
                            BlockEntityOnBlockReplacedMarker,
                        );
                    },
                    CreationType::Immediate,
                );
            }
            Self::__remove_block_entity(this.reclaim(), pos);
        }

        // Generic callback
        if !is_client
            && (old_state.block != state.block
                || {
                    bool::from(
                dsyn_instanceof!(cached this.dsyn_cache(), local_cx, &*state.block => export BlockAlwaysReplaceState)
                    .unwrap_or(default_block_always_replace_state())
                )
                })
            && (flags.contains(SetBlockStateFlags::NOTIFY_NEIGHBORS)
                || flags.contains(SetBlockStateFlags::MOVED))
            && let Some(server_w) = ServerWorld::downcast_arc_from_world(
                this.wca_as_wc()
                    .world_ptr
                    .upgrade()
                    .expect("world vanished"),
            )
        {
            let f = dsyn_instanceof!(cached this.dsyn_cache(), local_cx, &*old_state.block => export BlockOnStateReplaced<Cx>)
                .unwrap_or(default_block_on_state_replaced());
            f(
                old_state,
                &server_w,
                pos,
                flags.contains(SetBlockStateFlags::MOVED),
                local_cx,
                BlockOnStateReplacedMarker,
            );
        }

        let section = this.reclaim().bca().read_chunk_section(section_index)?;
        #[allow(clippy::if_then_some_else_none)] // too complex
        if section
            .block_state(pos_sec.x as u32, pos_sec.y as u32, pos_sec.z as u32)
            .block
            == state.block
        {
            drop(section);
            // Generic callback
            if !is_client && !flags.contains(SetBlockStateFlags::SKIP_BlOCK_ADDED_CALLBACK) {
                let f = dsyn_instanceof!(cached this.dsyn_cache(), local_cx, &*state.block => export BlockOnBlockAdded<Cx>)
                    .unwrap_or(default_block_on_block_added());
                f(
                    state,
                    old_state,
                    &this.wca_as_wc().world_ptr,
                    pos,
                    flags.contains(SetBlockStateFlags::MOVED),
                    local_cx,
                    BlockOnBlockAddedMarker,
                );
            }

            // Update block entity
            if let Some(be_constructor) = dsyn_instanceof!(cached this.dsyn_cache(), local_cx, &*state.block => export BlockEntityConstructor<Cx>)
            {
                #[derive(Clone, Copy)]
                enum PeekResult {
                    Update,
                    Remove,
                    Create,
                }

                let result = Self::__peek_block_entity_typed(
                    this.reclaim(),
                    pos,
                    |be| {
                        let bg = be.lock();
                        if bg.ty().erased_supports(state) {
                            #[cfg(feature = "tracing")]
                            tracing::warn!(
                                "found mismatched block entity {} at {pos} for block {}",
                                bg.ty(),
                                state.block
                            );

                            PeekResult::Update
                        } else {
                            PeekResult::Remove
                        }
                    },
                    CreationType::Check,
                )
                .unwrap_or(PeekResult::Create);

                match result {
                    PeekResult::Remove => {
                        let _be = Self::__remove_block_entity(this.reclaim(), pos);
                    }
                    PeekResult::Update => {
                        //TODO: set cached state for block entity
                        //TODO: update ticker of block entity
                    }
                    _ => {}
                }

                if matches!(result, PeekResult::Create | PeekResult::Remove) {
                    Self::__add_block_entity(
                        this.reclaim(),
                        be_constructor(pos, state, local_cx, BlockEntityConstructorMarker),
                    );
                }
            }

            //TODO: mark needs saving

            Some(old_state)
        } else {
            // Can this happen at end?
            None
        }
    }
}

impl<'r, 'w, Cx> AsBaseChunkAccess<'w, Cx> for &'r WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    type Access<'a>
        = &'r BaseChunk<'w, Cx>
    where
        Self: 'a;

    #[inline]
    fn as_base_chunk_access(&mut self) -> Self::Access<'_> {
        &self.base
    }

    #[inline]
    fn as_base_chunk(&self) -> &BaseChunk<'w, Cx> {
        &self.base
    }
}

impl<'w, Cx> AsBaseChunkAccess<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    type Access<'a>
        = &'a mut BaseChunk<'w, Cx>
    where
        Self: 'a;

    #[inline]
    fn as_base_chunk_access(&mut self) -> Self::Access<'_> {
        &mut self.base
    }

    #[inline]
    fn as_base_chunk(&self) -> &BaseChunk<'w, Cx> {
        &self.base
    }
}

impl<'w, Cx> WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
{
    fn __block_state(
        this: impl WorldChunkAccess<'w, Cx>,
        pos: BlockPos,
    ) -> Option<BlockState<'w, Cx>> {
        let index = this.wca_as_bc().height_limit.section_index(pos.y());
        this.bca().read_chunk_section(index).and_then(|section| {
            if section.is_empty() {
                None
            } else {
                let IVec3 { x, y, z } = pos.0 & (BORDER_LEN - 1) as i32;
                Some(section.block_state(x as u32, y as u32, z as u32))
            }
        })
    }

    fn __fluid_state(
        this: impl WorldChunkAccess<'w, Cx>,
        pos: BlockPos,
    ) -> Option<FluidState<'w, Cx>> {
        let bca = this.bca();
        let index = bca.bca_as_bc().height_limit.section_index(pos.y());
        bca.read_chunk_section(index).and_then(|section| {
            if section.is_empty() {
                None
            } else {
                let IVec3 { x, y, z } = pos.0 & (BORDER_LEN - 1) as i32;
                Some(section.fluid_state(x as u32, y as u32, z as u32))
            }
        })
    }
}

impl<'w, Cx> BlockView<'w, Cx> for &WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
{
    #[inline]
    fn block_state(&mut self, pos: BlockPos) -> Option<BlockState<'w, Cx>> {
        WorldChunk::__block_state(*self, pos)
    }

    #[inline]
    fn fluid_state(&mut self, pos: BlockPos) -> Option<FluidState<'w, Cx>> {
        WorldChunk::__fluid_state(*self, pos)
    }
}

impl<'w, Cx> BlockView<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
{
    #[inline]
    fn block_state(&mut self, pos: BlockPos) -> Option<BlockState<'w, Cx>> {
        WorldChunk::__block_state(self, pos)
    }

    #[inline]
    fn fluid_state(&mut self, pos: BlockPos) -> Option<FluidState<'w, Cx>> {
        WorldChunk::__fluid_state(self, pos)
    }
}

impl<'w, Cx> BlockEntityView<'w, Cx> for &WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    #[inline]
    fn peek_block_entity<F, T>(&mut self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T,
    {
        self.peek_block_entity_typed(pos, pk, CreationType::Check)
    }
}

impl<'w, Cx> BlockEntityView<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    #[inline]
    fn peek_block_entity<F, T>(&mut self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T,
    {
        self.peek_block_entity_typed_lf(pos, pk, CreationType::Check)
    }
}

impl<'w, Cx> BlockViewMut<'w, Cx> for &WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    #[inline]
    fn set_block_state(
        &mut self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
        flags: SetBlockStateFlags,
    ) -> Option<BlockState<'w, Cx>> {
        WorldChunk::__set_block_state(*self, pos, state, flags)
    }
}

impl<'w, Cx> BlockViewMut<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    #[inline]
    fn set_block_state(
        &mut self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
        flags: SetBlockStateFlags,
    ) -> Option<BlockState<'w, Cx>> {
        WorldChunk::__set_block_state(self, pos, state, flags)
    }
}

impl<'w, Cx> BlockEntityViewMut<'w, Cx> for &WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    #[inline]
    fn set_block_entity(&mut self, block_entity: Box<BlockEntity<'w, Cx>>) {
        WorldChunk::__set_block_entity(*self, block_entity, false);
    }

    #[inline]
    fn remove_block_entity(&mut self, pos: BlockPos) -> Option<BlockEntityCell<'w, Cx>> {
        WorldChunk::__remove_block_entity(*self, pos)
    }
}

impl<'w, Cx> BlockEntityViewMut<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    #[inline]
    fn set_block_entity(&mut self, block_entity: Box<BlockEntity<'w, Cx>>) {
        WorldChunk::__set_block_entity(self, block_entity, false);
    }

    #[inline]
    fn remove_block_entity(&mut self, pos: BlockPos) -> Option<BlockEntityCell<'w, Cx>> {
        WorldChunk::__remove_block_entity(self, pos)
    }
}

impl<'w, Cx> BlockLuminanceView<'w, Cx> for &WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    fn luminance(&mut self, pos: BlockPos) -> crate::view::StateOption<u32> {
        todo!()
    }
}

impl<'w, Cx> BlockLuminanceView<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    fn luminance(&mut self, pos: BlockPos) -> crate::view::StateOption<u32> {
        todo!()
    }
}

impl<'w, Cx> Chunk<'w, Cx> for &WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    #[inline]
    fn peek_game_event_dispatcher<F, T>(&mut self, y_section_coord: i32, f: F) -> Option<T>
    where
        F: for<'env> FnOnce(&'env Arc<game_event::Dispatcher<'w, Cx>>) -> T,
    {
        WorldChunk::__peek_game_event_dispatcher(&**self, y_section_coord, f)
    }
}

impl<'w, Cx> Chunk<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    #[inline]
    fn peek_game_event_dispatcher<F, T>(&mut self, y_section_coord: i32, f: F) -> Option<T>
    where
        F: for<'env> FnOnce(&'env Arc<game_event::Dispatcher<'w, Cx>>) -> T,
    {
        WorldChunk::__peek_game_event_dispatcher(self, y_section_coord, f)
    }
}

#[allow(unused)]
trait WorldChunkAccess<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    fn wca_as_wc(&self) -> &WorldChunk<'w, Cx>;
    fn wca_as_bc(&self) -> &BaseChunk<'w, Cx>;
    fn bca(self) -> impl BaseChunkAccess<'w, Cx>;

    fn reclaim(&mut self) -> impl WorldChunkAccess<'w, Cx>;
    unsafe fn reclaim_unsafe(&mut self) -> impl WorldChunkAccess<'w, Cx> + 'w;

    type GameEventDispatchersRead: Deref<
        Target = IHashMap<i32, Arc<game_event::Dispatcher<'w, Cx>>>,
    >;
    type GameEventDispatchersWrite: DerefMut<
        Target = IHashMap<i32, Arc<game_event::Dispatcher<'w, Cx>>>,
    >;

    fn read_game_event_dispatchers(self) -> Self::GameEventDispatchersRead;
    fn write_game_event_dispatchers(self) -> Self::GameEventDispatchersWrite;

    #[inline]
    fn dsyn_cache(&self) -> &DsynCache<'w, Cx> {
        &self.wca_as_wc().dsyn_cache
    }

    #[inline]
    fn local_cx(&self) -> Cx::LocalContext<'w> {
        self.wca_as_wc().local_cx
    }
}

impl<'a, 'w, Cx> WorldChunkAccess<'w, Cx> for &'a WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    unsafe fn reclaim_unsafe(&mut self) -> impl WorldChunkAccess<'w, Cx> + 'w {
        unsafe { &*std::ptr::from_ref(&**self) }
    }

    #[inline]
    fn wca_as_wc(&self) -> &WorldChunk<'w, Cx> {
        self
    }

    #[inline]
    fn wca_as_bc(&self) -> &BaseChunk<'w, Cx> {
        &self.base
    }

    #[inline]
    fn bca(self) -> impl BaseChunkAccess<'w, Cx> {
        &self.base
    }

    #[inline]
    fn reclaim(&mut self) -> impl WorldChunkAccess<'w, Cx> {
        *self
    }

    type GameEventDispatchersRead = Self::GameEventDispatchersWrite;
    type GameEventDispatchersWrite =
        MutexGuard<'a, IHashMap<i32, Arc<game_event::Dispatcher<'w, Cx>>>>;

    #[inline]
    fn read_game_event_dispatchers(self) -> Self::GameEventDispatchersRead {
        self.write_game_event_dispatchers()
    }

    #[inline]
    fn write_game_event_dispatchers(self) -> Self::GameEventDispatchersWrite {
        self.game_event_dispatchers.lock()
    }
}

impl<'a, 'w, Cx> WorldChunkAccess<'w, Cx> for &'a mut WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    unsafe fn reclaim_unsafe(&mut self) -> impl WorldChunkAccess<'w, Cx> + 'w {
        unsafe { &mut *std::ptr::from_mut(&mut **self) }
    }

    #[inline]
    fn wca_as_wc(&self) -> &WorldChunk<'w, Cx> {
        self
    }

    #[inline]
    fn wca_as_bc(&self) -> &BaseChunk<'w, Cx> {
        &self.base
    }

    #[inline]
    fn bca(self) -> impl BaseChunkAccess<'w, Cx> {
        &mut self.base
    }

    #[inline]
    fn reclaim(&mut self) -> impl WorldChunkAccess<'w, Cx> {
        &mut **self
    }

    type GameEventDispatchersRead = Self::GameEventDispatchersWrite;

    type GameEventDispatchersWrite = &'a mut IHashMap<i32, Arc<game_event::Dispatcher<'w, Cx>>>;

    #[inline]
    fn read_game_event_dispatchers(self) -> Self::GameEventDispatchersRead {
        self.write_game_event_dispatchers()
    }

    #[inline]
    fn write_game_event_dispatchers(self) -> Self::GameEventDispatchersWrite {
        self.game_event_dispatchers.get_mut()
    }
}
