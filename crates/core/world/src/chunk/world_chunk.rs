//! World chunks.

use dsyn::HoldDescriptors as _;
use ident_hash::IHashMap;
use local_cx::{LocalContext, dsyn_instanceof, dsyn_ty, dyn_cx::AsDynamicContext};
use parking_lot::Mutex;
use rimecraft_block::BlockState;
use rimecraft_block_entity::{
    BlockEntity, DynErasedRawBlockEntityType, component::RawErasedComponentType,
};
use rimecraft_fluid::{BsToFs, FluidState};
use rimecraft_registry::Registry;
use rimecraft_voxel_math::{BlockPos, IVec3};
use serde::{Deserialize, de::DeserializeSeed};

use crate::{
    DsynCache, Sealed, ServerWorld, World, behave::*, event::game_event, heightmap, view::block::*,
};

use super::{
    AsBaseChunk, AsBaseChunkMut, BORDER_LEN, BaseChunk, BlockEntityCell, Chunk, ChunkCx, ChunkMut,
    section::ComputeIndex,
};

use std::{
    fmt::Debug,
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
        + LocalContext<dsyn::Type<BlockOnBlockAdded<Cx>>>,
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
    Cx::BlockStateExt: Debug,
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
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    /// Peeks a [`BlockEntity`] at the target location, with given [`CreationType`].
    pub fn peek_block_entity_typed<F, T>(&self, pos: BlockPos, pk: F, ty: CreationType) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T,
    {
        let mut bes = self.base.block_entities.lock();
        let be = bes.get(&pos).cloned();
        if let Some(ref be) = be {
            if be.lock().is_removed() {
                bes.remove(&pos);
                return None;
            }
        } else {
            if let Some(nbt) = self.base.block_entity_nbts.lock().remove(&pos) {
                if let Some(be2) = self.load_block_entity_locked(pos, nbt) {
                    return Some(pk(&be2));
                }
            }
            if ty == CreationType::Immediate {
                if let Some(be) = self.create_block_entity(pos) {
                    self.add_block_entity_locked(be)
                }
            }
        }

        be.as_ref().map(pk)
    }

    /// Peeks a [`BlockEntity`] at the target location, with given [`CreationType`].
    pub fn peek_block_entity_typed_lf<F, T>(
        &mut self,
        pos: BlockPos,
        pk: F,
        ty: CreationType,
    ) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T,
    {
        let be = self.base.block_entities.get_mut().get(&pos).cloned();
        if let Some(ref be) = be {
            if be.lock().is_removed() {
                self.base.block_entities.get_mut().remove(&pos);
                return None;
            }
        } else {
            if let Some(nbt) = self.base.block_entity_nbts.get_mut().remove(&pos) {
                if let Some(be2) = self.load_block_entity(pos, nbt) {
                    return Some(pk(&be2));
                }
            }
            if ty == CreationType::Immediate {
                if let Some(be) = self.create_block_entity_lf(pos) {
                    self.add_block_entity(be)
                }
            }
        }

        be.as_ref().map(pk)
    }

    /// Adds a block entity to this chunk.
    pub fn add_block_entity(&mut self, block_entity: Box<BlockEntity<'w, Cx>>) {
        self.set_block_entity(block_entity);
        //TODO: Update tickers and game event listeners
    }

    /// Adds a block entity to this chunk.
    pub fn add_block_entity_locked(&self, block_entity: Box<BlockEntity<'w, Cx>>) {
        self.set_block_entity_locked(block_entity);
        //TODO: Update tickers and game event listeners
    }

    fn load_block_entity(
        &mut self,
        pos: BlockPos,
        nbt: Cx::Compound,
    ) -> Option<BlockEntityCell<'w, Cx>> {
        let be = DeserializeSeed::deserialize(
            rimecraft_block_entity::serde::Seed {
                pos,
                state: self.block_state_lf(pos).unwrap(),
                local_cx: self.local_cx,
            },
            Cx::compound_to_deserializer(&nbt),
        )
        .ok();

        if let Some(be) = be {
            self.add_block_entity(be);
            Some(
                self.base
                    .block_entities
                    .get_mut()
                    .get(&pos)
                    .expect("block entity should be inserted into this chunk")
                    .clone(),
            )
        } else {
            None
        }
    }

    fn load_block_entity_locked(
        &self,
        pos: BlockPos,
        nbt: Cx::Compound,
    ) -> Option<BlockEntityCell<'w, Cx>> {
        let be = DeserializeSeed::deserialize(
            rimecraft_block_entity::serde::Seed {
                pos,
                state: self.block_state(pos).unwrap(),
                local_cx: self.local_cx,
            },
            Cx::compound_to_deserializer(&nbt),
        )
        .ok();

        if let Some(be) = be {
            self.add_block_entity_locked(be);
            Some(
                self.base
                    .block_entities
                    .lock()
                    .get(&pos)
                    .expect("block entity should be inserted into this chunk")
                    .clone(),
            )
        } else {
            None
        }
    }

    #[inline]
    fn create_block_entity(&self, pos: BlockPos) -> Option<Box<BlockEntity<'w, Cx>>> {
        let bs = self.block_state(pos)?;
        dsyn_instanceof!(cached &*self.dsyn_cache, self.local_cx, &*bs.block => export BlockEntityConstructor<Cx>)
                .map(|f| f(pos, bs, self.local_cx, BlockEntityConstructorMarker))
    }

    #[inline]
    fn create_block_entity_lf(&mut self, pos: BlockPos) -> Option<Box<BlockEntity<'w, Cx>>> {
        let dsyn_ty =
            dsyn_ty!(cached &*self.dsyn_cache, self.local_cx => BlockEntityConstructor<Cx>);
        let local_cx = self.local_cx;
        let bs = self.block_state_lf(pos)?;
        (*bs.block)
            .descriptors()
            .get(dsyn_ty)
            .map(|f| f(pos, bs, local_cx, BlockEntityConstructorMarker))
    }
}

impl<'w, Cx> AsBaseChunk<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn as_base_chunk(&self) -> Sealed<&BaseChunk<'w, Cx>> {
        (&self.base).into()
    }
}

impl<'w, Cx> AsBaseChunkMut<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    #[inline]
    fn as_base_chunk_mut(&mut self) -> Sealed<&mut BaseChunk<'w, Cx>> {
        (&mut self.base).into()
    }
}

impl<'w, Cx> BlockView<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
{
    fn block_state(&self, pos: BlockPos) -> Option<BlockState<'w, Cx>> {
        self.base
            .section_array
            .get(self.base.height_limit.section_index(pos.y()))
            .and_then(|section| {
                let rg = section.lock();
                if rg.is_empty() {
                    None
                } else {
                    let IVec3 { x, y, z } = pos.0 & (BORDER_LEN - 1) as i32;
                    Some(rg.block_state(x as u32, y as u32, z as u32))
                }
            })
    }

    fn fluid_state(&self, pos: BlockPos) -> Option<FluidState<'w, Cx>> {
        self.base
            .section_array
            .get(self.base.height_limit.section_index(pos.y()))
            .and_then(|section| {
                let rg = section.lock();
                if rg.is_empty() {
                    None
                } else {
                    let IVec3 { x, y, z } = pos.0 & (BORDER_LEN - 1) as i32;
                    Some(rg.fluid_state(x as u32, y as u32, z as u32))
                }
            })
    }
}

impl<'w, Cx> BlockEntityView<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    #[inline(always)]
    fn peek_block_entity<F, T>(&self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T,
    {
        self.peek_block_entity_typed(pos, pk, CreationType::Check)
    }
}

impl<'w, Cx> LockFreeBlockView<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
{
    fn block_state_lf(&mut self, pos: BlockPos) -> Option<BlockState<'w, Cx>> {
        self.base
            .section_array
            .get_mut(self.base.height_limit.section_index(pos.y()))
            .and_then(|section| {
                let rg = section.get_mut();
                if rg.is_empty() {
                    None
                } else {
                    let IVec3 { x, y, z } = pos.0 & (BORDER_LEN - 1) as i32;
                    Some(rg.block_state(x as u32, y as u32, z as u32))
                }
            })
    }

    fn fluid_state_lf(&mut self, pos: BlockPos) -> Option<FluidState<'w, Cx>> {
        self.base
            .section_array
            .get_mut(self.base.height_limit.section_index(pos.y()))
            .and_then(|section| {
                let rg = section.get_mut();
                if rg.is_empty() {
                    None
                } else {
                    let IVec3 { x, y, z } = pos.0 & (BORDER_LEN - 1) as i32;
                    Some(rg.fluid_state(x as u32, y as u32, z as u32))
                }
            })
    }
}

impl<'w, Cx> LockFreeBlockEntityView<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    #[inline(always)]
    fn peek_block_entity_lf<F, T>(&mut self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T,
    {
        self.peek_block_entity_typed_lf(pos, pk, CreationType::Check)
    }
}

impl<'w, Cx> BlockViewMut<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    fn set_block_state(
        &mut self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
        flags: SetBlockStateFlags,
    ) -> Option<BlockState<'w, Cx>> {
        #[cfg(feature = "tracing")]
        let _span =
            tracing::trace_span!("set block state", pos = %pos, block = %state.block).entered();

        let section_index = self.height_limit().section_index(pos.y());
        let section = self.section_mut(section_index)?.get_mut();

        // Skip setting empty block (eg air) inside empty sections.
        if section.is_empty() && state.block.settings().is_empty {
            return None;
        }

        // Set state inside chunk section.
        let pos_sec = pos.0 & (BORDER_LEN as i32 - 1);
        let old_state =
            section.set_block_state(pos_sec.x as u32, pos_sec.y as u32, pos_sec.z as u32, state);

        // Aftermath

        // Skip handling identical state
        if std::ptr::eq(old_state.state, state.state) {
            return None;
        }

        // Update height maps
        // (vanilla) MOTION_BLOCKING,MOTION_BLOCKING_NO_LEAVES, OCEAN_FLOOR, OCEAN_FLOOR, WORLD_SURFACE.
        for ty in <Cx::HeightmapType as heightmap::Type<'w, Cx>>::iter_block_update_types_wc() {
            let this_ptr = self as *mut WorldChunk<'w, Cx>;
            if let Some(hm) = self.base.heightmaps.get_mut().get_mut(ty) {
                hm.track_update(pos_sec.x, pos.y(), pos_sec.z, state, |pos, pred| {
                    //SAFETY: This is safe because the `hms` is a valid pointer, and `peek_block_state_lf` does not interact with heightmaps.
                    pred(unsafe { &mut *this_ptr }.block_state_lf(pos))
                });
            }
        }

        //TODO: update chunk manager
        //TODO: update lighting

        // Remove old block entity if present
        if old_state.block != state.block
            && dsyn_instanceof!(cached &*self.dsyn_cache, self.local_cx, &*old_state.block => BlockEntityConstructor<Cx>)
        {
            if !self.is_client
                && !flags.contains(SetBlockStateFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK)
            {
                let ty = dsyn_ty!(cached &*self.dsyn_cache, self.local_cx => BlockEntityOnBlockReplaced<Cx>);
                let wa = self.world_ptr.clone();
                let local_cx = self.local_cx;

                self.peek_block_entity_typed_lf(
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
            self.remove_block_entity(pos);
        }

        if !self.is_client
            && (old_state.block != state.block
                || {
                    bool::from(
                dsyn_instanceof!(cached &*self.dsyn_cache, self.local_cx, &*state.block => export BlockAlwaysReplaceState)
                    .unwrap_or(default_block_always_replace_state())
                )
                })
            && (flags.contains(SetBlockStateFlags::NOTIFY_NEIGHBORS)
                || flags.contains(SetBlockStateFlags::MOVED))
            && let Some(server_w) = ServerWorld::downcast_arc_from_world(
                self.world_ptr.upgrade().expect("world vanished"),
            )
        {
            let f = dsyn_instanceof!(cached &*self.dsyn_cache, self.local_cx, &*old_state.block => export BlockOnStateReplaced<Cx>)
                .unwrap_or(default_block_on_state_replaced());
            f(
                old_state,
                &server_w,
                pos,
                flags.contains(SetBlockStateFlags::MOVED),
                self.local_cx,
                BlockOnStateReplacedMarker,
            );
        }

        let section = self.section_mut(section_index)?.get_mut();
        #[allow(clippy::if_then_some_else_none)] // too complex
        if section
            .block_state(pos_sec.x as u32, pos_sec.y as u32, pos_sec.z as u32)
            .block
            == state.block
        {
            if !self.is_client && !flags.contains(SetBlockStateFlags::SKIP_BlOCK_ADDED_CALLBACK) {
                let f = dsyn_instanceof!(cached &*self.dsyn_cache, self.local_cx, &*state.block => export BlockOnBlockAdded<Cx>)
                    .unwrap_or(default_block_on_block_added());
                f(
                    state,
                    old_state,
                    &self.world_ptr,
                    pos,
                    flags.contains(SetBlockStateFlags::MOVED),
                    self.local_cx,
                    BlockOnBlockAddedMarker,
                );
            }

            if let Some(be_constructor) = dsyn_instanceof!(cached &*self.dsyn_cache, self.local_cx, &*state.block => export BlockEntityConstructor<Cx>)
            {
                #[derive(Clone, Copy)]
                enum PeekResult {
                    Update,
                    Remove,
                    Create,
                }

                let result = self
                    .peek_block_entity_typed_lf(
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
                        let _be = self.remove_block_entity(pos);
                    }
                    PeekResult::Update => {
                        //TODO: set cached state for block entity
                        //TODO: update ticker of block entity
                    }
                    _ => {}
                }

                if matches!(result, PeekResult::Create | PeekResult::Remove) {
                    self.add_block_entity(be_constructor(
                        pos,
                        state,
                        self.local_cx,
                        BlockEntityConstructorMarker,
                    ));
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

impl<'w, Cx> BlockEntityViewMut<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    fn set_block_entity(&mut self, mut block_entity: Box<BlockEntity<'w, Cx>>) {
        let dsyn_ty =
            dsyn_ty!(cached &*self.dsyn_cache, self.local_cx => BlockEntityConstructor<Cx>);
        if self
            .block_state_lf(block_entity.pos())
            .map(|bs| (*bs.block).descriptors().contains(dsyn_ty))
            .unwrap_or_default()
        {
            block_entity.cancel_removal();
            let mut be2 = self
                .base
                .block_entities
                .get_mut()
                .insert(block_entity.pos(), Arc::new(Mutex::new(block_entity)));
            if let Some(be) = &mut be2 {
                if let Some(be) = Arc::get_mut(be) {
                    be.get_mut().mark_removed();
                } else {
                    be.lock().mark_removed();
                }
            }
        }
    }

    fn remove_block_entity(&mut self, pos: BlockPos) -> Option<BlockEntityCell<'w, Cx>> {
        if self.can_tick_block_entities() {
            let mut be = self.base.block_entities.get_mut().remove(&pos);
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
}

impl<'w, Cx> LockedBlockViewMut<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    fn set_block_state_locked(
        &self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
        flags: SetBlockStateFlags,
    ) -> Option<BlockState<'w, Cx>> {
        todo!()
    }

    fn set_block_entity_locked(&self, mut block_entity: Box<BlockEntity<'w, Cx>>) {
        block_entity.cancel_removal();
        let mut be2 = self
            .base
            .block_entities
            .lock()
            .insert(block_entity.pos(), Arc::new(Mutex::new(block_entity)));
        if let Some(be) = &mut be2 {
            be.lock().mark_removed();
        }
    }
}

impl<'w, Cx> LockedBlockEntityViewMut<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    fn remove_block_entity_locked(&self, pos: BlockPos) -> Option<BlockEntityCell<'w, Cx>> {
        todo!()
    }
}

impl<'w, Cx> BlockLuminanceView<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    fn luminance(&self, pos: BlockPos) -> crate::view::StateOption<u32> {
        todo!()
    }
}

impl<'w, Cx> Chunk<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    fn peek_game_event_dispatcher<F, T>(&self, y_section_coord: i32, f: F) -> Option<T>
    where
        F: for<'env> FnOnce(&'env Arc<game_event::Dispatcher<'w, Cx>>) -> T,
    {
        if self.is_client {
            None
        } else {
            let mut g = self.game_event_dispatchers.lock();
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
}

impl<'w, Cx> ChunkMut<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx> + AsDynamicContext,
{
    fn peek_game_event_dispatcher_lf<F, T>(&mut self, y_section_coord: i32, f: F) -> Option<T>
    where
        F: for<'env> FnOnce(&'env Arc<game_event::Dispatcher<'w, Cx>>) -> T,
    {
        if self.is_client {
            None
        } else {
            let g = self.game_event_dispatchers.get_mut();
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
}
