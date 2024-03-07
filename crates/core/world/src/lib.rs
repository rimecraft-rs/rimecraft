//! Minecraft world implementation.
//!
//! World generation is not implemented.
//!
//! # The World Lifetime
//!
//! The world lifetime is `'w`, in common. It is the lifetime of the world itself,
//! and `BlockState`s, `FluidState`s and the `Biome` registry should be bound to this lifetime.

use rimecraft_registry::Registry;

pub mod chunk;
pub mod tick;
pub mod view;

pub mod behave;

/// A wrapper for the `Registry` to adapt it to [`rimecraft_chunk_palette`] traits.
#[derive(Debug, Clone, Copy)]
pub struct RegPalWrapper<'a, K, T>(pub &'a Registry<K, T>);
