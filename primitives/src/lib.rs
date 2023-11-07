pub mod identifier;
pub mod reference;

#[cfg(feature = "serde")]
mod serde_update;

#[cfg(test)]
mod tests;

pub use identifier::Identifier as Id;
pub use reference::Reference as Ref;


#[cfg(feature = "serde")]
pub use serde_update::{Update as SerDeUpdate, ErasedUpdate as ErasedSerDeUpdate};

#[cfg(feature = "macros")]
pub use rimecraft_primitives_macros::*;
