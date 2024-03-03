//! Types and traits for working with chunks of blocks in a world.

use rimecraft_chunk_palette::container::ProvidePalette;

mod internal_types;
mod section;

pub use internal_types::*;
pub use section::ChunkSection;

/// Types associated with a [`ChunkSection`].
pub trait ChunkSectionTy<'bs, 'bi> {
    /// The type of block state extensions.
    type BlockStateExt: 'bs;
    /// The type of block state id list.
    type BlockStateList;

    /// The type of fluid state extensions.
    type FluidStateExt: 'bs;

    /// The type of biomes.
    type Biome: 'bi;
    /// The type of biome id list.
    type BiomeList;
}

/// Trait for computing the index of a position in a chunk section for [`PalettedContainer`].
pub trait ComputeIndex<L, T>: ProvidePalette<L, T> {
    /// Computes the index of the given position.
    ///
    /// The number type is unsigned because the index will overflow if it's negative.
    #[inline]
    fn compute_index(x: u32, y: u32, z: u32) -> usize {
        ((y << Self::EDGE_BITS | z) << Self::EDGE_BITS | x) as usize
    }
}
