//! Minecraft Block primitives.

use rimecraft_registry::Reg;

use std::marker::PhantomData;

mod pos;

pub use pos::BlockPos;

/// Block containing settings.
#[derive(Debug)]
pub struct RawBlock<P> {
    settings: Settings,
    _marker: PhantomData<P>,
}

impl<P> RawBlock<P> {
    /// Creates a new `Block` with the given settings.
    #[inline]
    pub const fn new(settings: Settings) -> Self {
        Self {
            settings,
            _marker: PhantomData,
        }
    }

    /// Returns the settings of the block.
    #[inline]
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}

impl<P> From<Settings> for RawBlock<P> {
    #[inline]
    fn from(settings: Settings) -> Self {
        Self::new(settings)
    }
}

/// A voxel in a `World`.
pub type Block<'r, K, P> = Reg<'r, K, RawBlock<P>>;

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

/// Represents a block state.
#[cfg(feature = "state")]
pub type BlockState<'s, 'r, K, P> = rimecraft_state::State<'s, Reg<'r, K, RawBlock<P>>>;
