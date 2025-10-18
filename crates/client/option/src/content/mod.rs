//! Various option content types.

use enum_iterator::Sequence;

mod attack_indicator;
mod cloud_render_mode;
mod graphics_mode;
mod inactivity_fps_limit;
mod narrator_mode;
mod particles_mode;
mod perspective;

pub use attack_indicator::*;
pub use cloud_render_mode::*;
pub use graphics_mode::*;
pub use inactivity_fps_limit::*;
pub use narrator_mode::*;
pub use particles_mode::*;
pub use perspective::*;

/// If a [`Sequence`], often enums, implements this, it will be allowed to get items directly through [`usize`] indexes. Wrapping behavior is configurable.
pub trait ByUSizeId: Sequence {
    /// Gets the [`usize`] id.
    fn get_usize_id(&self) -> Option<usize>
    where
        Self: PartialEq,
    {
        let all = enum_iterator::all::<Self>().collect::<Vec<_>>();
        all.iter().position(|value| value == self)
    }

    /// Gets an item by the specified [`usize`] index, where argument `wraps` specifies its wrapping behavior. If not exist, a [`None`] is retured.
    fn by_usize_id(id: usize, wraps: bool) -> Option<Self> {
        let mut all = enum_iterator::all::<Self>().collect::<Vec<_>>();
        let size = all.len();

        if wraps {
            let id = id % size;
            Some(all.remove(id))
        } else if id < size {
            Some(all.remove(id))
        } else {
            None
        }
    }

    /// Gets the next item. If not exist, the first item is returned.
    fn wrapping_next(&self) -> Self
    where
        Self: PartialEq,
    {
        self.next().unwrap_or(Self::first().unwrap())
    }
}
