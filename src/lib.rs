pub mod block;
pub mod entity;
pub mod fluid;
pub mod item;
/// Thin wrapper between Rimecraft modules
/// and [`fastnbt_rc`] and [`fastsnbt`].
pub mod nbt;
pub mod network;
/// Registry stuffs for managing almost all parts of in-game components.
pub mod registry;
pub mod server;
pub mod state;
mod util;
pub mod world;

pub use util::*;

/// Core utils of Rimecraft.
pub mod prelude {
    pub use crate::{
        nbt::NbtCompoundExt,
        util::{math::BlockPos, EnumValues, Id},
    };
}
