//! Voxel math library.

mod block_pos;
mod chunk_pos;
mod chunk_section_pos;

pub use block_pos::BlockPos;
pub use chunk_pos::ChunkPos;
pub use chunk_section_pos::ChunkSectionPos;

mod bbox;
pub mod direction;

pub use bbox::BBox;

pub use glam;

/// Converts a world coordinate to the corresponding chunk-section
/// coordinate.
///
/// This function is equivalent to `coord >> 4`.
/// To convert a floating-point coordinate, use [`section_coord_f64`].
#[inline]
pub const fn section_coord(coord: i32) -> i32 {
    coord >> 4
}

/// Converts a world coordinate to the corresponding chunk-section
/// coordinate.
///
/// This function is equivalent to `section_coord(coord.floor() as i32)`.
/// See [`section_coord`].
#[inline]
pub fn section_coord_f64(coord: f64) -> i32 {
    section_coord(coord.floor() as i32)
}
