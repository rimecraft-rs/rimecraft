//! Minecraft item primitives.

use std::{marker::PhantomData, num::NonZeroU32};

use rimecraft_fmt::Formatting;
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_registry::{ProvideRegistry, Reg};

pub mod stack;

pub use stack::ItemStack;

/// Item containing settings.
#[derive(Debug)]
pub struct RawItem<Cx> {
    settings: Settings,
    _marker: PhantomData<Cx>,
}

impl<Cx> RawItem<Cx> {
    /// Creates a new `Item` with the given settings.
    #[inline]
    pub const fn new(settings: Settings) -> Self {
        Self {
            settings,
            _marker: PhantomData,
        }
    }

    /// Returns the settings of the item.
    #[inline]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}

impl<Cx> From<Settings> for RawItem<Cx> {
    #[inline]
    fn from(settings: Settings) -> Self {
        Self::new(settings)
    }
}

impl<'r, K, Cx> ProvideRegistry<'r, K, Self> for RawItem<Cx>
where
    Cx: ProvideRegistry<'r, K, Self>,
{
    #[inline]
    fn registry() -> &'r rimecraft_registry::Registry<K, Self> {
        Cx::registry()
    }
}

/// An item usable by players and other entities.
pub type Item<'r, Cx> = Reg<'r, <Cx as ProvideIdTy>::Id, RawItem<Cx>>;

/// The max item count of an `ItemStack`.
pub const MAX_STACK_COUNT: u32 = 64;

/// Settings of an [`Item`].
///
/// A setting configure behaviors common to all items, such as the
/// stack's max count.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// The maximum count of the item that can be stacked in a single slot.
    pub max_count: NonZeroU32,
    /// The maximum amount of damage the item can take.
    pub max_damage: Option<NonZeroU32>,

    /// The rarity of the item.
    pub rarity: Rarity,
}

impl Default for Settings {
    #[inline]
    fn default() -> Self {
        Self {
            max_count: NonZeroU32::new(MAX_STACK_COUNT).unwrap(),
            max_damage: None,
            rarity: Default::default(),
        }
    }
}

#[doc(alias = "ItemProperties")]
pub use Settings as ItemSettings;

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
