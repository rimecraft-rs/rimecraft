//! World chunks.

use rimecraft_block::BlockState;
use rimecraft_block_entity::{BlockEntity, ProvideBlockEntity};
use rimecraft_fluid::{BsToFs, FluidState};
use rimecraft_voxel_math::{BlockPos, IVec3};

use crate::{
    view::block::{BlockView, BlockViewMut, LockFreeBlockView},
    Sealed,
};

use super::{section::ComputeIndex, AsBaseChunk, AsBaseChunkMut, BaseChunk, ChunkCx, BORDER_LEN};

use std::fmt::Debug;

/// Chunk for worlds.
pub struct WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// The [`BaseChunk`].
    pub base: BaseChunk<'w, Cx>,
    //TODO： The `World` pointer.
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
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
{
    pub fn peek_block_entity_typed<F, T>(&self, pos: BlockPos, pk: F, ty: CreationType) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntity<'w, Cx>) -> T,
    {
        let be = self.base.block_entities.get(&pos);
        todo!()
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
    fn peek_block_state<F, T>(&self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockState<'w, Cx>) -> T,
    {
        self.base
            .section_array
            .get(self.base.height_limit.section_index(pos.y()))
            .and_then(|section| {
                let rg = section.read();
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
                let rg = section.read();
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

    fn peek_block_entity<F, T>(&self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntity<'w, Cx>) -> T,
    {
        todo!()
    }
}

impl<'w, Cx> LockFreeBlockView<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
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

    fn peek_block_entity_lf<F, T>(&mut self, pos: BlockPos, pk: F) -> Option<T>
    where
        F: for<'s> FnOnce(&'s BlockEntity<'w, Cx>) -> T,
    {
        todo!()
    }
}

impl<'w, Cx> BlockViewMut<'w, Cx> for WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>,
    Cx::BlockStateExt: ProvideBlockEntity<Cx>,
{
    fn set_block_state(
        &mut self,
        pos: BlockPos,
        state: BlockState<'w, Cx>,
        moved: bool,
    ) -> Option<BlockState<'w, Cx>> {
        todo!()
    }

    fn set_block_entity(&mut self, mut block_entity: Box<BlockEntity<'w, Cx>>) {
        if self
            .peek_block_state_lf(block_entity.pos(), |bs| bs.state.data().has_block_entity())
            .unwrap_or_default()
        {
            //TODO: set world for block entity if necessary.
            block_entity.cancel_removal();
            let mut be2 = self
                .base
                .block_entities
                .insert(block_entity.pos(), block_entity);
            if let Some(be) = &mut be2 {
                be.mark_removed();
            }
        }
    }
}
