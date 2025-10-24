//! World chunks.

use dsyn::HoldDescriptors as _;
use glam::IVec3;
use local_cx::{LocalContext, dsyn_instanceof, dsyn_ty};
use parking_lot::Mutex;
use rimecraft_block::BlockState;
use rimecraft_block_entity::{
    BlockEntity, DynErasedRawBlockEntityType, component::RawErasedComponentType,
};
use rimecraft_fluid::{BsToFs, FluidState};
use rimecraft_global_cx::Hold;
use rimecraft_registry::Registry;
use rimecraft_voxel_math::BlockPos;
use serde::{Deserialize, de::DeserializeSeed};

use crate::{
    DsynCache, World,
    behave::*,
    chunk::{AsBaseChunkAccess, BaseChunkAccess, light::ChunkSkyLight},
    event::ServerChunkEventCallback,
    heightmap,
    view::{block::*, light::*},
};

use super::{BORDER_LEN, BaseChunk, BlockEntityCell, Chunk, ChunkCx, section::ComputeIndex};

use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, Weak},
};

/// Local context bounds alias for most [`WorldChunk`] operations.
pub trait WorldChunkLocalCx<'w, Cx>:
    LocalContext<&'w Registry<Cx::Id, RawErasedComponentType<'w, Cx>>>
    + LocalContext<&'w Registry<Cx::Id, DynErasedRawBlockEntityType<'w, Cx>>>
    + LocalContext<dsyn::Type<BlockEntityConstructor<Cx>>>
where
    Cx: ChunkCx<'w>,
{
}

impl<'w, Cx, L> WorldChunkLocalCx<'w, Cx> for L
where
    Cx: ChunkCx<'w>,
    L: LocalContext<&'w Registry<Cx::Id, RawErasedComponentType<'w, Cx>>>
        + LocalContext<&'w Registry<Cx::Id, DynErasedRawBlockEntityType<'w, Cx>>>
        + LocalContext<dsyn::Type<BlockEntityConstructor<Cx>>>,
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
    world_ptr: Weak<World<'w, Cx>>,

    local_cx: Cx::LocalContext<'w>,
    dsyn_cache: Arc<DsynCache<'w, Cx>>,

    ext: Cx::WorldChunkExt,

    _invariant_marker: PhantomData<fn(&'w ()) -> &'w ()>,
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
    /// Returns the weak world pointer of this chunk.
    #[inline]
    pub fn world_ptr(&self) -> &Weak<World<'w, Cx>> {
        &self.world_ptr
    }

    /// Returns the extension of this chunk.
    #[inline]
    pub fn ext(&self) -> &Cx::WorldChunkExt {
        &self.ext
    }

    /// Returns the extension of this chunk mutably.
    #[inline]
    pub fn ext_mut(&mut self) -> &mut Cx::WorldChunkExt {
        &mut self.ext
    }

    #[inline]
    fn can_tick_block_entities(&self) -> bool {
        self.loaded_to_world || self.is_client
    }

    #[inline]
    fn __mark_needs_saving(this: impl WorldChunkAccess<'w, Cx>) {
        this.bca().mark_needs_saving();
        //TODO: call dirty chunk listener
    }

    /// Downcasts a [`Chunk`] to a [`WorldChunk`].
    ///
    /// # Errors
    ///
    /// Returns the chunk if it is not a [`WorldChunk`], or it is **a reference of it.**
    #[inline]
    pub fn downcast<C: Chunk<'w, Cx>>(value: C) -> Result<Self, C> {
        //SAFETY: 'w is invariant here. won't produce lifetime soundness issues
        unsafe { rcutil::try_cast(value) }
    }

    /// Downcasts a [`Chunk`] to a [`WorldChunk`] by reference.
    ///
    /// # Errors
    ///
    /// Returns the chunk if it is not a [`WorldChunk`].
    #[inline]
    pub fn downcast_ref<C: Chunk<'w, Cx>>(value: &C) -> Option<&Self> {
        unsafe {
            //SAFETY: 'w is invariant here. won't produce lifetime soundness issues
            rcutil::try_cast::<_, &&Self>(value)
                .copied()
                .or_else(|value| rcutil::try_cast::<_, &Self>(value))
                .ok()
        }
    }
}

impl<'w, Cx> WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
{
    #[doc(hidden)]
    pub fn __peek_block_entity_typed<F, T, W>(
        mut this: W,
        pos: BlockPos,
        pk: F,
        ty: CreationType,
    ) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T,
        W: WorldChunkAccess<'w, Cx>,
        Cx: ServerChunkEventCallback<'w, W>,
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
                && let Some(be2) = Self::__load_block_entity(
                    //SAFETY: because of restrictions of Rust's borrow checker, the only way to get through this complex type system is through this.
                    // due to the anonymous lifetime this is safe to do. they will do nothing to the extended lifetime.
                    unsafe { this.reclaim_unsafe() },
                    pos,
                    nbt,
                )
            {
                return Some(pk(&be2));
            }
            if ty == CreationType::Immediate
                && let Some(be) = Self::__create_block_entity(this.reclaim(), pos)
            {
                Self::__add_block_entity(this, be)
            }
        }

        be.as_ref().map(pk)
    }

    /// Peeks a [`BlockEntity`] at the target location, with given [`CreationType`].
    #[inline]
    pub fn peek_block_entity_typed<'a, F, T>(
        &'a self,
        pos: BlockPos,
        pk: F,
        ty: CreationType,
    ) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T,
        Cx: ServerChunkEventCallback<'w, &'a Self>,
    {
        Self::__peek_block_entity_typed(self, pos, pk, ty)
    }

    /// Peeks a [`BlockEntity`] at the target location, with given [`CreationType`].
    #[inline]
    pub fn peek_block_entity_typed_lf<'a, F, T>(
        &'a mut self,
        pos: BlockPos,
        pk: F,
        ty: CreationType,
    ) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntityCell<'w, Cx>) -> T,
        Cx: ServerChunkEventCallback<'w, &'a mut Self>,
    {
        Self::__peek_block_entity_typed(self, pos, pk, ty)
    }

    fn __add_block_entity<W>(mut this: W, block_entity: Box<BlockEntity<'w, Cx>>)
    where
        W: WorldChunkAccess<'w, Cx>,
        Cx: ServerChunkEventCallback<'w, W>,
    {
        if let Some(cell) = Self::__set_block_entity(this.reclaim(), block_entity, true) {
            Cx::add_block_entity_callback(&cell, &mut this);
            //PLACEHOLDER
            let _ = ();
            //TODO: Update tickers
        }
    }

    /// Adds a block entity to this chunk.
    #[inline]
    pub fn add_block_entity<'a>(&'a mut self, block_entity: Box<BlockEntity<'w, Cx>>)
    where
        Cx: ServerChunkEventCallback<'w, &'a mut Self>,
    {
        Self::__add_block_entity(self, block_entity);
    }

    /// Adds a block entity to this chunk.
    #[inline]
    pub fn add_block_entity_locked<'a>(&'a self, block_entity: Box<BlockEntity<'w, Cx>>)
    where
        Cx: ServerChunkEventCallback<'w, &'a Self>,
    {
        Self::__add_block_entity(self, block_entity);
    }

    fn __load_block_entity<W>(
        mut this: W,
        pos: BlockPos,
        nbt: Cx::Compound,
    ) -> Option<BlockEntityCell<'w, Cx>>
    where
        W: WorldChunkAccess<'w, Cx>,
        Cx: ServerChunkEventCallback<'w, W>,
    {
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
            let res = this
                .reclaim()
                .bca()
                .read_block_entities()
                .get(&pos)
                .expect("block entity should be inserted into this chunk")
                .clone();
            Self::__add_block_entity(this, be);
            Some(res)
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
        let bs_w = Self::__block_state(this.reclaim(), block_entity.pos())
            .filter(|bs| (*bs.block).descriptors().contains(dsyn_ty))?;

        // State checks
        let bs_be = block_entity.cached_state();
        if bs_w != bs_be {
            if !block_entity.ty().erased_supports(bs_w) {
                return None; // In-world block state does not support this block entity
            }
            // Update cached state to in-world state
            block_entity.set_cached_state(bs_w);
        }

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
    }

    fn __remove_block_entity<W>(mut this: W, pos: BlockPos) -> Option<BlockEntityCell<'w, Cx>>
    where
        W: WorldChunkAccess<'w, Cx>,
        Cx: ServerChunkEventCallback<'w, W>,
    {
        if this.wca_as_wc().can_tick_block_entities() {
            let mut be = this.reclaim().bca().write_block_entities().remove(&pos);
            if let Some(be) = &mut be {
                Cx::remove_block_entity_callback(be, &mut this);
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

    fn __set_block_state<W>(
        mut this: W,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
        flags: SetBlockStateFlags,
    ) -> Option<BlockState<'w, Cx>>
    where
        W: WorldChunkAccess<'w, Cx>,
        Cx: ServerChunkEventCallback<'w, W>,
    {
        #[cfg(feature = "tracing")]
        let _span =
            tracing::trace_span!("set block state", pos = %pos, block = %state.block).entered();

        let local_cx = this.local_cx();

        let section_index = this.wca_as_bc().height_limit.section_index(pos.y());
        let mut section = this.reclaim().bca().write_chunk_section(section_index)?;

        // Skip setting empty block (eg air) inside empty sections.
        if section.is_empty() && state.block.settings().is_empty {
            return None;
        }

        // Set state inside chunk section.
        let pos_sec = pos.0.as_uvec3() % BORDER_LEN; // first four bits. TODO: make this follow BORDER_LEN constant
        let old_state = section.set_block_state(pos_sec.x, pos_sec.y, pos_sec.z, state);

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
                hm.track_update(
                    pos_sec.x as i32,
                    pos.y(),
                    pos_sec.z as i32,
                    state,
                    |pos, pred| pred(Self::__block_state(this_short_life.reclaim(), pos)),
                );
            }
        }

        //TODO: update chunk manager
        //TODO: update lighting

        // Generic callbacks
        Cx::replace_block_state_callback(pos, state, old_state, flags, &mut this);

        if old_state.block != state.block
            && dsyn_instanceof!(cached this.dsyn_cache(), local_cx, &*old_state.block => BlockEntityConstructor<Cx>)
        {
            Self::__remove_block_entity(
                //SAFETY: because of restrictions of Rust's borrow checker, the only way to get through this complex type system is through this.
                // due to the anonymous lifetime this is safe to do. they will do nothing to the extended lifetime.
                unsafe { this.reclaim_unsafe() },
                pos,
            );
        }

        let section = this.reclaim().bca().read_chunk_section(section_index)?;
        #[allow(clippy::if_then_some_else_none)] // too complex
        if section.block_state(pos_sec.x, pos_sec.y, pos_sec.z).block == state.block {
            drop(section);
            // Generic callback
            Cx::add_block_state_callback(pos, state, old_state, flags, &mut this);

            // Update block entity
            if let Some(be_constructor) = dsyn_instanceof!(cached this.dsyn_cache(), local_cx, &*state.block => export BlockEntityConstructor<Cx>)
            {
                #[derive(Clone, Copy, PartialEq, Eq)]
                enum PeekResult {
                    Update,
                    Remove,
                    Create,
                }

                let result = Self::__peek_block_entity_typed(
                    //SAFETY: because of restrictions of Rust's borrow checker, the only way to get through this complex type system is through this.
                    // due to the anonymous lifetime this is safe to do. they will do nothing to the extended lifetime.
                    unsafe { this.reclaim_unsafe() },
                    pos,
                    |be| {
                        let mut bg = be.lock();
                        if bg.ty().erased_supports(state) {
                            #[cfg(feature = "tracing")]
                            tracing::warn!(
                                "found mismatched block entity {} at {pos} for block {}",
                                bg.ty(),
                                state.block
                            );

                            bg.set_cached_state(state);
                            //TODO: update ticker of block entity

                            PeekResult::Update
                        } else {
                            PeekResult::Remove
                        }
                    },
                    CreationType::Check,
                )
                .unwrap_or(PeekResult::Create);

                if result == PeekResult::Remove {
                    //SAFETY: because of restrictions of Rust's borrow checker, the only way to get through this complex type system is through this.
                    // due to the anonymous lifetime this is safe to do. they will do nothing to the extended lifetime.
                    let _be = Self::__remove_block_entity(unsafe { this.reclaim_unsafe() }, pos);
                }

                if matches!(result, PeekResult::Create | PeekResult::Remove) {
                    Self::__add_block_entity(
                        //SAFETY: same as above
                        unsafe { this.reclaim_unsafe() },
                        be_constructor(pos, state, local_cx, BlockEntityConstructorMarker),
                    );
                }
            }

            Self::__mark_needs_saving(this.reclaim());
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

    fn __luminance(this: impl WorldChunkAccess<'w, Cx>, pos: BlockPos) -> u32 {
        Self::__block_state(this, pos)
            .map(|state| state.data().get_held().luminance())
            .unwrap_or(0)
    }

    #[inline]
    fn __peek_chunk_sky_light<F, U>(this: impl WorldChunkAccess<'w, Cx>, f: F) -> U
    where
        F: FnOnce(&ChunkSkyLight) -> U,
    {
        f(&this.bca().read_chunk_sky_light())
    }

    fn __light_sources(
        this: &mut impl Chunk<'w, Cx>,
    ) -> impl Iterator<Item = (BlockPos, BlockState<'w, Cx>)> {
        this.blocks()
            .filter(|(_, bs)| bs.data().get_held().luminance() > 0)
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
    Cx: ChunkCx<'w>
        + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>
        + BsToFs<'w>
        + ServerChunkEventCallback<'w, Self>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
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
    Cx: ChunkCx<'w>
        + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>
        + BsToFs<'w>
        + for<'a> ServerChunkEventCallback<'w, &'a mut Self>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
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
    Cx: ChunkCx<'w>
        + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>
        + BsToFs<'w>
        + ServerChunkEventCallback<'w, Self>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
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
    Cx: ChunkCx<'w>
        + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>
        + BsToFs<'w>
        + for<'a> ServerChunkEventCallback<'w, &'a mut Self>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
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
    Cx: ChunkCx<'w>
        + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>
        + BsToFs<'w>
        + ServerChunkEventCallback<'w, Self>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
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
    Cx: ChunkCx<'w>
        + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>
        + BsToFs<'w>
        + for<'a> ServerChunkEventCallback<'w, &'a mut Self>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
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
    Cx: ChunkCx<'w>
        + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>
        + BsToFs<'w>
        + ServerChunkEventCallback<'w, Self>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
{
    #[inline]
    fn luminance(&mut self, pos: BlockPos) -> u32 {
        WorldChunk::__luminance(*self, pos)
    }
}

impl<'w, Cx> BlockLuminanceView<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
{
    #[inline]
    fn luminance(&mut self, pos: BlockPos) -> u32 {
        WorldChunk::__luminance(self, pos)
    }
}

impl<'w, Cx> LightSourceView<'w, Cx> for &WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>
        + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>
        + BsToFs<'w>
        + ServerChunkEventCallback<'w, Self>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
{
    #[inline]
    fn peek_chunk_sky_light<F, U>(&mut self, f: F) -> U
    where
        F: FnOnce(&ChunkSkyLight) -> U,
    {
        WorldChunk::__peek_chunk_sky_light(*self, f)
    }

    #[inline]
    fn light_sources(&mut self) -> impl Iterator<Item = (BlockPos, BlockState<'w, Cx>)> {
        WorldChunk::__light_sources(self)
    }
}

impl<'w, Cx> LightSourceView<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>
        + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>
        + BsToFs<'w>
        + for<'a> ServerChunkEventCallback<'w, &'a mut Self>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
{
    #[inline]
    fn peek_chunk_sky_light<F, U>(&mut self, f: F) -> U
    where
        F: FnOnce(&ChunkSkyLight) -> U,
    {
        WorldChunk::__peek_chunk_sky_light(self, f)
    }

    #[inline]
    fn light_sources(&mut self) -> impl Iterator<Item = (BlockPos, BlockState<'w, Cx>)> {
        WorldChunk::__light_sources(self)
    }
}

impl<'w, Cx> Chunk<'w, Cx> for &WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>
        + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>
        + BsToFs<'w>
        + ServerChunkEventCallback<'w, Self>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
{
}

impl<'w, Cx> Chunk<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>
        + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>
        + BsToFs<'w>
        + for<'a> ServerChunkEventCallback<'w, &'a mut Self>,
    Cx::Id: for<'de> Deserialize<'de>,
    Cx::LocalContext<'w>: WorldChunkLocalCx<'w, Cx>,
{
}

#[allow(missing_docs)]
pub trait WorldChunkAccess<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    fn wca_as_wc(&self) -> &WorldChunk<'w, Cx>;
    fn wca_as_bc(&self) -> &BaseChunk<'w, Cx>;
    fn bca(self) -> impl BaseChunkAccess<'w, Cx>;

    type Reclaim<'a>: WorldChunkAccess<'w, Cx>
    where
        Self: 'a;

    fn reclaim(&mut self) -> Self::Reclaim<'_>;

    // this is safe if the reclaimed value is only used by external functions that
    // will not care about the what exactly is the given lifetime.
    /// # Safety
    ///
    /// This copies a mutable reference.
    unsafe fn reclaim_unsafe(&mut self) -> Self;

    #[inline]
    fn dsyn_cache(&self) -> &DsynCache<'w, Cx> {
        &self.wca_as_wc().dsyn_cache
    }

    #[inline]
    fn local_cx(&self) -> Cx::LocalContext<'w> {
        self.wca_as_wc().local_cx
    }
}

impl<'w, Cx> WorldChunkAccess<'w, Cx> for &WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    type Reclaim<'e>
        = &'e WorldChunk<'w, Cx>
    where
        Self: 'e;

    #[inline]
    unsafe fn reclaim_unsafe(&mut self) -> Self {
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
    fn reclaim(&mut self) -> Self::Reclaim<'_> {
        *self
    }
}

impl<'w, Cx> WorldChunkAccess<'w, Cx> for &mut WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    type Reclaim<'e>
        = &'e mut WorldChunk<'w, Cx>
    where
        Self: 'e;

    #[inline]
    unsafe fn reclaim_unsafe(&mut self) -> Self {
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
    fn reclaim(&mut self) -> Self::Reclaim<'_> {
        &mut **self
    }
}
