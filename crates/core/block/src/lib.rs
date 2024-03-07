//! Minecraft Block primitives.

use rimecraft_registry::{ProvideRegistry, Reg};
use rimecraft_state::{States, StatesMut};

use std::marker::PhantomData;

pub use rimecraft_state as state;

/// Block containing settings and the state manager.
#[derive(Debug)]
pub struct RawBlock<'a, SExt, Cx> {
    settings: Settings,
    states: States<'a, SExt>,
    _marker: PhantomData<Cx>,
}

impl<'a, SExt, Cx> RawBlock<'a, SExt, Cx> {
    /// Creates a new block with the given settings.
    #[inline]
    pub const fn new(settings: Settings, states: States<'a, SExt>) -> Self {
        Self {
            settings,
            states,
            _marker: PhantomData,
        }
    }

    /// Returns the settings of the block.
    #[inline]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Returns the state manager of the block.
    #[inline]
    pub fn states(&self) -> &States<'a, SExt> {
        &self.states
    }
}

impl<SExt, Cx> From<Settings> for RawBlock<'_, SExt, Cx>
where
    SExt: Default + Clone,
{
    #[inline]
    fn from(settings: Settings) -> Self {
        Self::new(settings, StatesMut::new(Default::default()).freeze())
    }
}

impl<'r, SExt, K, Cx> ProvideRegistry<'r, K, Self> for RawBlock<'r, SExt, Cx>
where
    Cx: ProvideRegistry<'r, K, Self>,
{
    #[inline(always)]
    fn registry() -> &'r rimecraft_registry::Registry<K, Self> {
        Cx::registry()
    }
}

/// A voxel in a `World`.
pub type Block<'a, K, SExt, Cx> = Reg<'a, K, RawBlock<'a, SExt, Cx>>;

/// Settings of a block.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Settings {
    /// Whether this block can be collided with.
    pub collidable: bool,
    /// The resistance of this block.
    pub resistance: f32,
    /// The hardness of this block.
    pub hardness: f32,
    /// Whether this block accepts random ticks.
    pub random_ticks: bool,
    /// Whether this block is empty.
    #[doc(alias = "is_air")]
    pub is_empty: bool,
    /// Whether this block is opaque.
    pub opaque: bool,
}

#[doc(alias = "BlockProperties")]
pub use Settings as BlockSettings;

/// Global contexts providing global `BlockState` IDs.
///
/// # MCJE Reference
///
/// This is the equivalent of `net.minecraft.block.Block.STATE_IDS` in MCJE.
pub trait ProvideStateIds {
    /// The type of the state IDs.
    type List;

    /// Returns the state IDs.
    fn state_ids() -> Self::List;
}
