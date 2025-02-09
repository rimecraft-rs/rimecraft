//! Minecraft Fluid primitives.

use rimecraft_block::{BlockState, ProvideBlockStateExtTy};
use rimecraft_global_cx::ProvideIdTy;
use rimecraft_maybe::Maybe;
use rimecraft_registry::Reg;
use rimecraft_state::{State, States};

use std::{fmt::Debug, hash::Hash, marker::PhantomData, sync::Arc};

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

/// Settings of a fluid.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Settings {
    /// Whether this fluid accepts random ticks.
    pub random_ticks: bool,
    /// Whether this fluid is empty.
    pub is_empty: bool,
}

/// A fluid in a `World`.
pub type Fluid<'a, Cx> = Reg<'a, <Cx as ProvideIdTy>::Id, RawFluid<'a, Cx>>;

/// Global contexts providing fluid state extensions.
pub trait ProvideFluidStateExtTy: ProvideIdTy {
    /// The type of the fluid state extension.
    type FluidStateExt;
}

/// The `FluidState` type.
///
/// This contains the fluid registration and the [`State`].
pub struct FluidState<'w, Cx>
where
    Cx: ProvideFluidStateExtTy,
{
    /// The fluid.
    pub fluid: Fluid<'w, Cx>,
    /// The state.
    pub state: Arc<State<'w, Cx::FluidStateExt>>,
}

impl<Cx> Debug for FluidState<'_, Cx>
where
    Cx: ProvideFluidStateExtTy + Debug,
    Cx::Id: Debug,
    Cx::FluidStateExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IFluidState")
            .field("fluid", &self.fluid)
            .field("state", &self.state)
            .finish()
    }
}

impl<Cx> Clone for FluidState<'_, Cx>
where
    Cx: ProvideFluidStateExtTy,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            fluid: self.fluid,
            state: self.state.clone(),
        }
    }
}

impl<Cx> PartialEq for FluidState<'_, Cx>
where
    Cx: ProvideFluidStateExtTy,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.fluid == other.fluid && Arc::ptr_eq(&self.state, &other.state)
    }
}

impl<Cx> Eq for FluidState<'_, Cx> where Cx: ProvideFluidStateExtTy {}

impl<Cx> Hash for FluidState<'_, Cx>
where
    Cx: ProvideFluidStateExtTy,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fluid.hash(state);
        Arc::as_ptr(&self.state).hash(state);
    }
}

/// Global Contexts that is able to convert [`BlockState`] to [`FluidState`] instances.
pub trait BsToFs<'w>
where
    Self: ProvideFluidStateExtTy + ProvideBlockStateExtTy,
{
    /// Converts a block state to a fluid state.
    fn block_to_fluid_state<'a>(
        bs: Maybe<'a, BlockState<'w, Self>>,
    ) -> Maybe<'a, FluidState<'w, Self>>;
}

/// Extenstions to the `Maybe<'_, IBlockState<'_, _>>`.
pub trait BlockStateExt<'a, 'w, Cx>
where
    Cx: ProvideFluidStateExtTy,
{
    /// Converts this block state to fluid state.
    fn to_fluid_state(self) -> Maybe<'a, FluidState<'w, Cx>>;
}

impl<'a, 'w, Cx> BlockStateExt<'a, 'w, Cx> for Maybe<'a, BlockState<'w, Cx>>
where
    Cx: BsToFs<'w>,
{
    #[inline]
    fn to_fluid_state(self) -> Maybe<'a, FluidState<'w, Cx>> {
        Cx::block_to_fluid_state(self)
    }
}

impl<'a, 'w, Cx> BlockStateExt<'a, 'w, Cx> for &'a BlockState<'w, Cx>
where
    Cx: BsToFs<'w>,
{
    #[inline]
    fn to_fluid_state(self) -> Maybe<'a, FluidState<'w, Cx>> {
        Cx::block_to_fluid_state(Maybe::Borrowed(self))
    }
}
