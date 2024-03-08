//! Rimecraft global context traits.
//!
//! # What is a global context?
//!
//! A global context (`Cx` in convention), is a global types and behavior provider for the whole
//! game. It provides types and behaviors that are used across the whole game, such as the
//! identifier type, the palette provider, and so on.

#![no_std]

use core::fmt::Display;

/// Marker trait for global contexts.
pub trait GlobalContext: 'static {}

/// Marker trait for global contexts that provide an identifier type.
pub trait ProvideIdTy: GlobalContext {
    /// Identifier type.
    ///
    /// [`Display`] is required for error handling purposes.
    type Identifier: Display;
}
