pub mod item;
/// Thin wrapper between Minecraft code structure and [`fastnbt`] and [`fastsnbt`].
pub mod nbt;
/// Registry stuffs for managing almost all parts of in-game components.
pub mod registry;
pub mod server;
pub mod util;

pub mod prelude {
    pub use crate::{registry::RegistryKey, util::Identifier};
}
