pub mod block;
pub mod component;
pub mod entity;
pub mod fluid;
pub mod item;
/// Thin wrapper between Rimecraft modules
/// and [`fastnbt`] and [`fastsnbt`].
pub mod nbt;
pub mod net;
/// Registry stuffs for managing almost all parts of in-game components.
pub mod registry;
pub mod server;
pub mod state;
pub mod text;
mod util;
pub mod world;

pub use util::*;

/// Core types of Rimecraft.
pub mod prelude {
    pub use crate::{
        nbt::NbtCompoundExt,
        util::{math::BlockPos, EnumValues, Id},
    };
}
