//! Minecraft Item primitives.

use std::{marker::PhantomData, num::NonZeroU32};

use rimecraft_fmt::Formatting;
use rimecraft_registry::{ProvideRegistry, Reg};

pub mod stack;

pub use stack::ItemStack;

/// Item containing settings.
#[derive(Debug)]
pub struct RawItem<P> {
    settings: Settings,
    _marker: PhantomData<P>,
}

impl<P> RawItem<P> {
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

impl<P> From<Settings> for RawItem<P> {
    #[inline]
    fn from(settings: Settings) -> Self {
        Self::new(settings)
    }
}

impl<'r, K, P> ProvideRegistry<'r, K, Self> for RawItem<P>
where
    P: ProvideRegistry<'r, K, Self>,
{
    #[inline]
    fn registry() -> &'r rimecraft_registry::Registry<K, Self> {
        P::registry()
    }
}

/// An item usable by players and other entities.
pub type Item<'r, K, P> = Reg<'r, K, RawItem<P>>;

/// A trait for converting a value to an [`Item`].
#[doc(alias = "ItemConvertible")]
#[doc(alias = "ItemLike")]
pub trait ToItem<'s, 'r, K, P> {
    /// Converts the value to an [`Item`].
    fn to_item(&'s self) -> Item<'r, K, P>;
}

impl<'ss, 's, 'r, K, P, T> ToItem<'ss, 'r, K, P> for &'s T
where
    T: ToItem<'s, 'r, K, P>,
{
    #[inline]
    fn to_item(&'ss self) -> Item<'r, K, P> {
        (*self).to_item()
    }
}

impl<'r, K, P> ToItem<'_, 'r, K, P> for Item<'r, K, P> {
    #[inline]
    fn to_item(&'_ self) -> Item<'r, K, P> {
        *self
    }
}

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