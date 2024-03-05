//! Minecraft world implementation.
//!
//! World generation is not implemented.
//!
//! # Design
//!
//! ## World lifetime
//!
//! The world lifetime is `'w`. It is the lifetime of the world itself,
//! and block states and the `Biome` registry should be bound to this lifetime.

pub mod chunk;
pub mod tick;
pub mod view;

pub mod behave;
