//! World chunks.

use rimecraft_block::BlockState;
use rimecraft_block_entity::BlockEntity;
use rimecraft_fluid::{BsToFs, FluidState};
use rimecraft_voxel_math::{BlockPos, IVec3};

use crate::{view::BlockView, Sealed};

use super::{section::ComputeIndex, AsBaseChunk, AsBaseChunkMut, BaseChunk, ChunkCx, BORDER_LEN};

use std::fmt::Debug;

/// Chunk for worlds.
pub struct WorldChunk<'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// The [`BaseChunk`].
    pub base: BaseChunk<'w, Cx>,
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

impl<'w, Cx> WorldChunk<'w, Cx> where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>> + BsToFs<'w>
{
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
