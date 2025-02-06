//! Minecraft item primitives.

use std::marker::PhantomData;

use component::map::ComponentMap;
use rimecraft_fmt::Formatting;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::Reg;

#[cfg(feature = "edcode")]
mod edcode;
pub mod stack;

pub use component;
pub use stack::ItemStack;

/// Provides settings type for items.
pub trait ProvideSettingsTy: ProvideIdTy {
    /// Settings type of an item.
    type Settings<'a>: ItemSettings<'a, Self>;
}

/// Settings of an item.
pub trait ItemSettings<'a, Cx>
where
    Cx: ProvideIdTy,
{
    /// Returns components of the item.
    fn components(&self) -> &ComponentMap<'a, Cx>;
}

/// Item containing settings.
#[derive(Debug)]
pub struct RawItem<'a, Cx>
where
    Cx: ProvideSettingsTy,
{
    settings: Cx::Settings<'a>,
    _marker: PhantomData<Cx>,
}

impl<'a, Cx> RawItem<'a, Cx>
where
    Cx: ProvideSettingsTy,
{
    /// Creates a new `Item` with the given settings.
    #[inline]
    pub const fn new(settings: Cx::Settings<'a>) -> Self {
        Self {
            settings,
            _marker: PhantomData,
        }
    }

    /// Returns the settings of the item.
    #[inline]
    pub fn settings(&self) -> &Cx::Settings<'a> {
        &self.settings
    }
}

/// An item usable by players and other entities.
pub type Item<'r, Cx> = Reg<'r, <Cx as ProvideIdTy>::Id, RawItem<'r, Cx>>;

/// The max item count of an `ItemStack`.
pub const MAX_STACK_COUNT: u32 = 64u32;

/// Rarity of an item.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
#[non_exhaustive]
pub enum Rarity {
    /// Common rarity.
    #[default]
    Common,
    /// Uncommon rarity.
    Uncommon,
    /// Rare rarity.
    Rare,
    /// Epic rarity.
    Epic,
}

impl From<Rarity> for Formatting {
    #[inline]
    fn from(value: Rarity) -> Self {
        match value {
            Rarity::Common => Formatting::White,
            Rarity::Uncommon => Formatting::Yellow,
            Rarity::Rare => Self::Aqua,
            Rarity::Epic => Self::LightPurple,
        }
    }
}
