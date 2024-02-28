//! Types and traits for working with chunks of blocks in a world.

use rimecraft_block::{Block, BlockState};
use rimecraft_chunk_palette::{
    container::{PalettedContainer, ProvidePalette},
    IndexFromRaw as PalIndexFromRaw,
};
use rimecraft_registry::Reg;
use rimecraft_state::State;
use rimecraft_voxel_math::{BlockPos, IVec3};

type InnerBs<'bs, K, Cx> = BlockState<'bs, K, Cx>;
type InnerBi<'bi, K, Bi> = Reg<'bi, K, Bi>;

/// Section on a `Chunk`.
#[derive(Debug)]
pub struct ChunkSection<'bs, 'bi, BsL, Bi, BiL, K, Cx> {
    bsc: PalettedContainer<BsL, InnerBs<'bs, K, Cx>, Cx>,
    bic: PalettedContainer<BiL, InnerBi<'bi, K, Bi>, Cx>,

    ne_block_c: u16,
    rt_block_c: u16,
    ne_fluid_c: u16,
}

impl<'bs, 'bi, BsL, Bi, BiL, K, Cx> ChunkSection<'bs, 'bi, BsL, Bi, BiL, K, Cx> {
    /// Creates a new chunk section.
    #[inline]
    pub const fn new(
        bsc: PalettedContainer<BsL, State<'bs>, Cx>,
        bic: PalettedContainer<BiL, Reg<'bi, K, Bi>, Cx>,
    ) -> Self {
        Self {
            bsc,
            bic,
            ne_block_c: 0,
            rt_block_c: 0,
            ne_fluid_c: 0,
        }
    }
}

impl<'bs, 'bi, BsL, Bi, BiL, K, Cx> ChunkSection<'bs, 'bi, BsL, Bi, BiL, K, Cx>
where
    BsL: for<'s> PalIndexFromRaw<'s, &'s InnerBs<'bs, K, Cx>>,
    Cx: ComputeIndex<BsL, InnerBs<'bs, K, Cx>>,
{
    /// Gets the block state at the given position.
    #[inline]
    pub fn block_state(&self, pos: BlockPos) -> Option<(Block<'bs, K, Cx>, &State<'bs>)> {
        self.bsc.get(Cx::compute_index(pos)).map(|(b, s)| (*b, s))
    }

    pub fn fluid_state(&self, pos: BlockPos) -> Option<&Reg<'bi, K, Bi>> {
        self.bic.get(Cx::compute_index(pos))
    }
}

/// Trait for computing the index of a position in a chunk section for [`PalettedContainer`].
pub trait ComputeIndex<L, T>: ProvidePalette<L, T> {
    /// Computes the index of the given position.
    #[inline]
    fn compute_index(pos: BlockPos) -> usize {
        let BlockPos(IVec3 { x, y, z }) = pos;
        ((y << Self::EDGE_BITS | z) << Self::EDGE_BITS | x) as usize
    }
}
