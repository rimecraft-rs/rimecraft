//! Minecraft Block primitives.

use rimecraft_registry::Reg;
use rimecraft_state::{States, StatesMut};

use std::marker::PhantomData;

pub use rimecraft_state as state;

/// Block containing settings and the state manager.
#[derive(Debug)]
pub struct RawBlock<'a, P> {
    settings: Settings,
    states: States<'a>,
    _marker: PhantomData<P>,
}

impl<'a, P> RawBlock<'a, P> {
    /// Creates a new block with the given settings.
    #[inline]
    pub const fn new(settings: Settings, states: States<'a>) -> Self {
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
    pub fn states(&self) -> &States<'a> {
        &self.states
    }
}

impl<P> From<Settings> for RawBlock<'_, P> {
    #[inline]
    fn from(settings: Settings) -> Self {
        Self::new(settings, StatesMut::new().freeze())
    }
}

/// A voxel in a `World`.
pub type Block<'a, K, P> = Reg<'a, K, RawBlock<'a, P>>;

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
    /// Whether this block is air.
    pub is_air: bool,
    /// Whether this block is opaque.
    pub opaque: bool,
}

#[doc(alias = "BlockProperties")]
pub use Settings as BlockSettings;
