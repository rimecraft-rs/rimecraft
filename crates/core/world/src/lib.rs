//! Minecraft world implementation.
//!
//! World generation is not implemented.
//!
//! # The World Lifetime
//!
//! The world lifetime is `'w`, in common. It is the lifetime of the world itself,
//! and `BlockState`s, `FluidState`s and the `Biome` registry should be bound to this lifetime.

use chunk::{ChunkCx, IBlockState, IFluidState};
use rimecraft_chunk_palette::Maybe;

pub mod chunk;
pub mod tick;
pub mod view;

pub mod behave;

/// Global Contexts that is able to convert [`IBlockState`] to [`IFluidState`] instances.
pub trait BsToFs<'w>: ChunkCx<'w> {
    /// Converts a block state to a fluid state.
    fn block_to_fluid_state<'a>(
        bs: Maybe<'a, IBlockState<'w, Self>>,
    ) -> Maybe<'a, IFluidState<'w, Self>>;
}

/// Extenstions to the `Maybe<'_, IBlockState<'_, _>>`.
pub trait BlockStateExt<'a, 'w, Cx>
where
    Cx: ChunkCx<'w>,
{
    /// Converts this block state to fluid state.
    fn to_fluid_state(self) -> Maybe<'a, IFluidState<'w, Cx>>;
}

impl<'a, 'w, Cx> BlockStateExt<'a, 'w, Cx> for Maybe<'a, IBlockState<'w, Cx>>
where
    Cx: BsToFs<'w>,
{
    #[inline]
    fn to_fluid_state(self) -> Maybe<'a, IFluidState<'w, Cx>> {
        Cx::block_to_fluid_state(self)
    }
}

impl<'a, 'w, Cx> BlockStateExt<'a, 'w, Cx> for &'a IBlockState<'w, Cx>
where
    Cx: BsToFs<'w>,
{
    #[inline]
    fn to_fluid_state(self) -> Maybe<'a, IFluidState<'w, Cx>> {
        Cx::block_to_fluid_state(Maybe::Borrowed(self))
    }
}
