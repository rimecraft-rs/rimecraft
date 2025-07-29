//! World chunks.

use dsyn::HoldDescriptors as _;
use ident_hash::IHashMap;
use local_cx::{LocalContext, dsyn_instanceof, dsyn_ty, dyn_cx::AsDynamicContext};
use parking_lot::Mutex;
use rimecraft_block::BlockState;
use rimecraft_block_entity::{
    BlockEntity, BlockEntityConstructor, DynRawBlockEntityType, component::RawErasedComponentType,
};
use rimecraft_chunk_palette::{Maybe, SimpleOwned};
use rimecraft_fluid::{BsToFs, FluidState};
use rimecraft_registry::Registry;
use rimecraft_voxel_math::{BlockPos, IVec3};
use serde::{Deserialize, de::DeserializeSeed};

use crate::{
    DsynCache, Sealed,
    event::game_event,
    heightmap,
    view::block::{
        BlockLuminanceView, BlockView, BlockViewMut, LockFreeBlockView, LockedBlockViewMut,
        SetBlockStateFlags,
    },
};

use super::{
    AsBaseChunk, AsBaseChunkMut, BORDER_LEN, BaseChunk, BlockEntityCell, Chunk, ChunkCx, ChunkMut,
    section::ComputeIndex,
};

use std::{fmt::Debug, sync::Arc};

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
    Cx::LocalContext<'w>: LocalContext<&'w Registry<Cx::Id, RawErasedComponentType<'w, Cx>>>
        + LocalContext<&'w Registry<Cx::Id, DynRawBlockEntityType<'w, Cx>>>
        + LocalContext<dsyn::Type<BlockEntityConstructor<Cx>>>
        + AsDynamicContext,
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
                state: self.peek_block_state_lf(pos, BlockState::clone).unwrap(),
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
                state: self.peek_block_state(pos, BlockState::clone).unwrap(),
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
        self.peek_block_state(pos, |&bs| {
            dsyn_instanceof!(cached &*self.dsyn_cache, self.local_cx, &*bs.block => export BlockEntityConstructor<Cx>)
                .map(|f| f(pos, bs, self.local_cx))
        })
        .flatten()
    }

    #[inline]
    fn create_block_entity_lf(&mut self, pos: BlockPos) -> Option<Box<BlockEntity<'w, Cx>>> {
        let dsyn_ty =
            dsyn_ty!(cached &*self.dsyn_cache, self.local_cx => BlockEntityConstructor<Cx>);
        let local_cx = self.local_cx;
        self.peek_block_state_lf(pos, |&bs| {
            (*bs.block)
                .descriptors()
                .get(dsyn_ty)
                .map(|f| f(pos, bs, local_cx))
        })
        .flatten()
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
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: LocalContext<&'w Registry<Cx::Id, RawErasedComponentType<'w, Cx>>>
        + LocalContext<&'w Registry<Cx::Id, DynRawBlockEntityType<'w, Cx>>>
        + LocalContext<dsyn::Type<BlockEntityConstructor<Cx>>>
        + AsDynamicContext,
{
    fn peek_block_state<F, T>(&self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockState<'w, Cx>) -> T,
    {
        self.base
            .section_array
            .get(self.base.height_limit.section_index(pos.y()))
            .and_then(|section| {
                let rg = section.lock();
                if rg.is_empty() {
                    None
                } else {
                    let IVec3 { x, y, z } = pos.0 & (BORDER_LEN - 1) as i32;
                    rg.block_state(x as u32, y as u32, z as u32)
                        .as_deref()
                        .map(pk)
                }
            })
    }

    fn peek_fluid_state<F, T>(&self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s FluidState<'w, Cx>) -> T,
    {
        self.base
            .section_array
            .get(self.base.height_limit.section_index(pos.y()))
            .and_then(|section| {
                let rg = section.lock();
                if rg.is_empty() {
                    None
                } else {
                    let IVec3 { x, y, z } = pos.0 & (BORDER_LEN - 1) as i32;
                    rg.fluid_state(x as u32, y as u32, z as u32)
                        .as_deref()
                        .map(pk)
                }
            })
    }

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
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: LocalContext<&'w Registry<Cx::Id, RawErasedComponentType<'w, Cx>>>
        + LocalContext<&'w Registry<Cx::Id, DynRawBlockEntityType<'w, Cx>>>
        + LocalContext<dsyn::Type<BlockEntityConstructor<Cx>>>
        + AsDynamicContext,
{
    fn peek_block_state_lf<F, T>(&mut self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockState<'w, Cx>) -> T,
    {
        self.base
            .section_array
            .get_mut(self.base.height_limit.section_index(pos.y()))
            .and_then(|section| {
                let rg = section.get_mut();
                if rg.is_empty() {
                    None
                } else {
                    let IVec3 { x, y, z } = pos.0 & (BORDER_LEN - 1) as i32;
                    rg.block_state(x as u32, y as u32, z as u32)
                        .as_deref()
                        .map(pk)
                }
            })
    }

    fn peek_fluid_state_lf<F, T>(&mut self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s FluidState<'w, Cx>) -> T,
    {
        self.base
            .section_array
            .get_mut(self.base.height_limit.section_index(pos.y()))
            .and_then(|section| {
                let rg = section.get_mut();
                if rg.is_empty() {
                    None
                } else {
                    let IVec3 { x, y, z } = pos.0 & (BORDER_LEN - 1) as i32;
                    rg.fluid_state(x as u32, y as u32, z as u32)
                        .as_deref()
                        .map(pk)
                }
            })
    }

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
    Cx::LocalContext<'w>: LocalContext<&'w Registry<Cx::Id, RawErasedComponentType<'w, Cx>>>
        + LocalContext<&'w Registry<Cx::Id, DynRawBlockEntityType<'w, Cx>>>
        + LocalContext<dsyn::Type<BlockEntityConstructor<Cx>>>
        + AsDynamicContext,
{
    fn set_block_state(
        &mut self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
        flags: SetBlockStateFlags,
    ) -> Option<BlockState<'w, Cx>> {
        let section = self
            .section_mut(self.height_limit().section_index(pos.y()))?
            .get_mut();
        let sec_is_empty = section.is_empty();
        if sec_is_empty && state.block.settings().is_empty {
            return None;
        }

        let bs;
        let pos_alt = pos.0 & (BORDER_LEN as i32 - 1);
        {
            let IVec3 { x, y, z } = pos_alt;
            bs = section.set_block_state(x as u32, y as u32, z as u32, state);
        }

        if bs.is_some_and(|s| std::ptr::eq(s.state, state.state)) {
            return None;
        }

        {
            let IVec3 { x, y, z } = IVec3 {
                y: pos.y(),
                ..pos_alt
            };
            let this_ptr = self as *mut WorldChunk<'w, Cx>;
            // (vanilla) MOTION_BLOCKING,MOTION_BLOCKING_NO_LEAVES, OCEAN_FLOOR, OCEAN_FLOOR, WORLD_SURFACE.
            for ty in <Cx::HeightmapType as heightmap::Type<'w, Cx>>::iter_block_update_types_wc() {
                //SAFETY: This is safe because the `hms` is a valid pointer, and `peek_block_state_lf` does not interact with heightmaps.
                unsafe {
                    if let Some(hm) = self.base.heightmaps.get_mut().get_mut(ty) {
                        hm.track_update(x, y, z, &state, |pos, pred| {
                            (*this_ptr)
                                .peek_block_state_lf(pos, |bs| pred(Some(bs)))
                                .unwrap_or_else(|| pred(None))
                        });
                    }
                }
            }
        }

        //TODO: update chunk manager
        //TODO: update lighting
        //TODO: update profiler

        //TODO: lots to do here

        if let Some(bs) = bs {
            let has_be = dsyn_instanceof!(cached &*self.dsyn_cache, self.local_cx, &*bs.block => BlockEntityConstructor<Cx>);
            if !self.is_client {
                //TODO: call `on_state_replaced`.
            } else if bs.block != state.block && has_be {
                self.remove_block_entity(pos);
            }
        }

        todo!()
    }

    fn set_block_entity(&mut self, mut block_entity: Box<BlockEntity<'w, Cx>>) {
        let dsyn_ty =
            dsyn_ty!(cached &*self.dsyn_cache, self.local_cx => BlockEntityConstructor<Cx>);
        if self
            .peek_block_state_lf(block_entity.pos(), |bs| {
                (*bs.block).descriptors().contains(dsyn_ty)
            })
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
    Cx::LocalContext<'w>: LocalContext<&'w Registry<Cx::Id, RawErasedComponentType<'w, Cx>>>
        + LocalContext<&'w Registry<Cx::Id, DynRawBlockEntityType<'w, Cx>>>
        + LocalContext<dsyn::Type<BlockEntityConstructor<Cx>>>
        + AsDynamicContext,
{
    fn set_block_state_locked(
        &self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
        moved: bool,
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

    fn remove_block_entity_locked(&self, pos: BlockPos) -> Option<BlockEntityCell<'w, Cx>> {
        todo!()
    }
}

impl<'w, Cx> BlockLuminanceView<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: LocalContext<&'w Registry<Cx::Id, RawErasedComponentType<'w, Cx>>>
        + LocalContext<&'w Registry<Cx::Id, DynRawBlockEntityType<'w, Cx>>>
        + LocalContext<dsyn::Type<BlockEntityConstructor<Cx>>>
        + AsDynamicContext,
{
    fn luminance(&self, pos: BlockPos) -> crate::view::StateOption<u32> {
        todo!()
    }
}

impl<'w, Cx> Chunk<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: LocalContext<&'w Registry<Cx::Id, RawErasedComponentType<'w, Cx>>>
        + LocalContext<&'w Registry<Cx::Id, DynRawBlockEntityType<'w, Cx>>>
        + LocalContext<dsyn::Type<BlockEntityConstructor<Cx>>>
        + AsDynamicContext,
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
    Cx::LocalContext<'w>: LocalContext<&'w Registry<Cx::Id, RawErasedComponentType<'w, Cx>>>
        + LocalContext<&'w Registry<Cx::Id, DynRawBlockEntityType<'w, Cx>>>
        + LocalContext<dsyn::Type<BlockEntityConstructor<Cx>>>
        + AsDynamicContext,
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
