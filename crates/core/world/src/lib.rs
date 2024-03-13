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
