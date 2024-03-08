//! Minecraft Block primitives.

use rimecraft_global_cx::{GlobalContext, ProvideIdTy};
use rimecraft_registry::{ProvideRegistry, Reg};
use rimecraft_state::States;

use std::marker::PhantomData;

pub use rimecraft_state as state;

/// Fluid containing settings and the state manager.
#[derive(Debug)]
pub struct RawFluid<'a, Cx>
where
    Cx: ProvideFluidStateExtTy,
{
    states: States<'a, Cx::FluidStateExt>,
    settings: Settings,
    _marker: PhantomData<Cx>,
}

impl<'a, Cx> RawFluid<'a, Cx>
where
    Cx: ProvideFluidStateExtTy,
{
    /// Creates a new fluid from the given settings.
    #[inline]
    pub const fn new(settings: Settings, states: States<'a, Cx::FluidStateExt>) -> Self {
        Self {
            states,
            settings,
            _marker: PhantomData,
        }
    }

    /// Returns the state manager of the fluid.
    #[inline]
    pub fn states(&self) -> &States<'a, Cx::FluidStateExt> {
        &self.states
    }

    /// Returns the settings of the fluid.
    #[inline]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}

impl<'r, K, Cx> ProvideRegistry<'r, K, Self> for RawFluid<'r, Cx>
where
    Cx: ProvideFluidStateExtTy + ProvideRegistry<'r, K, Self>,
{
    #[inline(always)]
    fn registry() -> &'r rimecraft_registry::Registry<K, Self> {
        Cx::registry()
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
pub type Fluid<'a, Cx> = Reg<'a, <Cx as ProvideIdTy>::Identifier, RawFluid<'a, Cx>>;

/// Global contexts providing fluid state extensions.
pub trait ProvideFluidStateExtTy: GlobalContext {
    /// The type of the fluid state extension.
    type FluidStateExt;
}
