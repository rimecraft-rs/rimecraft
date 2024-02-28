//! Minecraft Block primitives.

use rimecraft_registry::Reg;
use rimecraft_state::{States, StatesMut};
use state::State;

use std::marker::PhantomData;

pub use rimecraft_state as state;

/// Block containing settings and the state manager.
#[derive(Debug)]
pub struct RawBlock<'a, Cx> {
    settings: Settings,
    states: States<'a>,
    _marker: PhantomData<Cx>,
}

impl<'a, Cx> RawBlock<'a, Cx> {
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

impl<Cx> From<Settings> for RawBlock<'_, Cx> {
    #[inline]
    fn from(settings: Settings) -> Self {
        Self::new(settings, StatesMut::new().freeze())
    }
}

/// A voxel in a `World`.
pub type Block<'a, K, Cx> = Reg<'a, K, RawBlock<'a, Cx>>;

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

struct ShapeCache {}
