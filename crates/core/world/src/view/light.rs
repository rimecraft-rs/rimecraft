//! Light view traits.

use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_fluid::ProvideFluidStateExtTy;
use rimecraft_voxel_math::BlockPos;

use crate::{DEFAULT_MAX_LIGHT_LEVEL, chunk::light::ChunkSkyLight, view::block::BlockView};

/// View of block luminance source levels.
pub trait BlockLuminanceView<'w, Cx>: BlockView<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
{
    /// Returns the luminance source level of the given position.
    fn luminance(&mut self, pos: BlockPos) -> u32;

    /// Returns the max light level of this view.
    ///
    /// The default one is [`DEFAULT_MAX_LIGHT_LEVEL`].
    #[inline]
    fn max_light_level(&mut self) -> u32 {
        DEFAULT_MAX_LIGHT_LEVEL
    }
}

/// View of light sources in a chunk.
pub trait LightSourceView<'w, Cx>: BlockView<'w, Cx>
where
    Cx: ProvideBlockStateExtTy + ProvideFluidStateExtTy,
{
    /// Peeks a [`ChunkSkyLight`] in a immutable manner.
    fn peek_chunk_sky_light<F, U>(&mut self, f: F) -> U
    where
        F: FnOnce(&ChunkSkyLight) -> U;

    /// Returns an iterator over the light sources in the chunk.
    fn light_sources(&mut self) -> impl Iterator<Item = (BlockPos, BlockState<'w, Cx>)>;
}
