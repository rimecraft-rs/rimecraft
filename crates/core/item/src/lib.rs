//! Minecraft item primitives.

use std::{marker::PhantomData, num::NonZeroU32};

use component::map::ComponentMap;
use rimecraft_fmt::Formatting;
use rimecraft_global_cx::{GlobalContext, ProvideIdTy};
use rimecraft_registry::{ProvideRegistry, Reg};

#[cfg(feature = "edcode")]
mod edcode;
pub mod stack;

pub use stack::ItemStack;

/// Provides settings type for items.
pub trait ProvideSettingsTy: GlobalContext {
    /// Settings type of an item.
    type Settings<'a>: ItemSettings<'a, Self>;
}

/// Settings of an item.
pub trait ItemSettings<'a, Cx> {
    /// Returns the base settings of the item.
    fn base(&self) -> &BaseSettings;

    /// Returns the *base components* of the item.
    fn components(&self) -> &'a ComponentMap<'a, Cx>
    where
        Cx: ProvideIdTy;
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

impl<'r, K, Cx> ProvideRegistry<'r, K, Self> for RawItem<'r, Cx>
where
    Cx: ProvideRegistry<'r, K, Self> + ProvideSettingsTy,
{
    #[inline]
    fn registry() -> &'r rimecraft_registry::Registry<K, Self> {
        Cx::registry()
    }
}

/// An item usable by players and other entities.
pub type Item<'r, Cx> = Reg<'r, <Cx as ProvideIdTy>::Id, RawItem<'r, Cx>>;

/// The max item count of an `ItemStack`.
pub const MAX_STACK_COUNT: u32 = 64;

/// Base settings of an [`Item`].
///
/// A setting configure behaviors common to all items, such as the
/// stack's max count.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseSettings {
    /// The maximum count of the item that can be stacked in a single slot.
    pub max_count: NonZeroU32,
    /// The maximum amount of damage the item can take.
    pub max_damage: Option<NonZeroU32>,

    /// The rarity of the item.
    pub rarity: Rarity,

    /// Whether an item should have its NBT data sent to the client.
    pub sync_nbt: bool,
}

impl Default for BaseSettings {
    #[inline]
    fn default() -> Self {
        Self {
            max_count: NonZeroU32::new(MAX_STACK_COUNT).unwrap(),
            max_damage: None,
            rarity: Default::default(),
            sync_nbt: true,
        }
    }
}
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
