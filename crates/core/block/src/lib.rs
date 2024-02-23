//! Minecraft Block primitives.

use rimecraft_registry::Reg;
use rimecraft_state::StatesMut;

use std::marker::PhantomData;

mod pos;

pub use pos::BlockPos;

/// Block containing settings and the state manager.
#[derive(Debug)]
pub struct RawBlock<'a, P> {
    settings: Settings,
    states: rimecraft_state::States<'a>,
    _marker: PhantomData<P>,
}

impl<'a, P> RawBlock<'a, P> {
    /// Creates a new block with the given settings.
    #[inline]
    pub fn new<B>(settings: Settings, p: B) -> Self
    where
        B: AppendProperties<'a>,
    {
        let mut states = StatesMut::new();
        p.append_properties(&mut states);
        Self {
            settings,
            states: states.freeze(),
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
    pub fn states(&self) -> &rimecraft_state::States<'a> {
        &self.states
    }
}

impl<P> From<Settings> for RawBlock<'_, P> {
    #[inline]
    fn from(settings: Settings) -> Self {
        Self::new(settings, ())
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

/// Types that can appends properties to the block.
pub trait AppendProperties<'a> {
    /// Appends properties to the block.
    fn append_properties(self, settings: &mut StatesMut<'a>);
}

impl AppendProperties<'_> for () {
    #[inline(always)]
    fn append_properties(self, _: &mut StatesMut<'_>) {}
}

impl<'a, F> AppendProperties<'a> for F
where
    F: for<'s> FnOnce(&'s mut StatesMut<'a>),
{
    #[inline]
    fn append_properties(self, settings: &mut StatesMut<'a>) {
        self(settings)
    }
}
