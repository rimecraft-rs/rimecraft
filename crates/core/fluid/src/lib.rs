//! Minecraft Block primitives.

use rimecraft_registry::Reg;
use rimecraft_state::States;

use std::marker::PhantomData;

pub use rimecraft_state as state;

/// Fluid containing settings and the state manager.
#[derive(Debug)]
pub struct RawFluid<'a, SExt, Cx> {
    states: States<'a, SExt>,
    settings: Settings,
    _marker: PhantomData<Cx>,
}

impl<'a, SExt, Cx> RawFluid<'a, SExt, Cx> {
    /// Creates a new fluid from the given settings.
    #[inline]
    pub const fn new(settings: Settings, states: States<'a, SExt>) -> Self {
        Self {
            states,
            settings,
            _marker: PhantomData,
        }
    }

    /// Returns the state manager of the fluid.
    #[inline]
    pub fn states(&self) -> &States<'a, SExt> {
        &self.states
    }

    /// Returns the settings of the fluid.
    #[inline]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}

/// Settings of a fluid.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Settings {
    /// Whether this fluid accepts random ticks.
    pub random_ticks: bool,
    /// Whether this fluid is empty.
    pub is_empty: bool,
}

/// A fluid in a `World`.
pub type Fluid<'a, K, SExt, Cx> = Reg<'a, K, RawFluid<'a, SExt, Cx>>;
