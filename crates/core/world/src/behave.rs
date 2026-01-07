//! Traits for representing behaviors of in-game components.
//!
//! Types listed there are usually used as descriptor types.

pub use rimecraft_block_entity::BlockEntityConstructorMarker;

/// Constructor of a block entity.
///
/// # Parameters
///
/// 1. Position of the block entity to construct.
/// 2. State of the block the block entity will be placed.
pub type BlockEntityConstructor<Cx> = rimecraft_block_entity::BlockEntityConstructor<Cx>;
