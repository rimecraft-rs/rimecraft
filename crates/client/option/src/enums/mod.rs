use enum_iterator::Sequence;

pub mod attack_indicator;
pub mod cloud_render_mode;
pub mod graphics_mode;
pub mod narrator_mode;
pub mod particles_mode;
pub mod perspective;

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
        } else {
            if id < size {
                Some(all.remove(id))
            } else {
                None
            }
        }
    }
}
