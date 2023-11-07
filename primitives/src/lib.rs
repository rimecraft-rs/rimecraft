pub mod identifier;
pub mod reference;

#[cfg(feature = "serde")]
mod serde_update;

#[cfg(test)]
mod tests;

pub use identifier::Identifier as Id;
pub use reference::Reference as Ref;

#[cfg(feature = "serde")]
pub use serde_update::{ErasedUpdate as ErasedSerDeUpdate, Update as SerDeUpdate};
