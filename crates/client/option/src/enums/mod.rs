use enum_iterator::Sequence;

pub mod attack_indicator;
pub mod cloud_render_mode;
pub mod graphics_mode;
pub mod narrator_mode;
pub mod particles_mode;
pub mod perspective;

/// If a [`Sequence`], often enums, implements this, it will be allowed to get items directly through [`i32`] indexes. Wrapping behavior is configurable.
pub trait ByIntId: Sequence {
    /// Gets an item by the specified [`i32`] index, where argument `wraps` specifies its wrapping behavior. If not exist, a [`None`] is retured.
    fn by_i32_id(id: i32, wraps: bool) -> Option<Self> {
        let mut all = enum_iterator::all::<Self>().collect::<Vec<_>>();
        let size = all.len() as i32;

        if wraps {
            let id = (id % size + size) % size;
            Some(all.remove(id as usize))
        } else {
            if id >= 0 && id < size {
                Some(all.remove(id as usize))
            } else {
                None
            }
        }
    }
}
