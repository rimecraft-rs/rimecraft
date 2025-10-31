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

/// Converts a world coordinate to the corresponding chunk-section coordinate.
///
/// This function is equivalent to `coord >> 4`.
/// To convert a floating-point coordinate, use [`coord_section_from_block_f64`].
#[inline]
#[doc(alias = "section_coord")]
pub const fn coord_section_from_block(coord: i32) -> i32 {
    coord >> 4
}

/// Converts a world coordinate to the corresponding chunk-section coordinate.
///
/// This function is equivalent to `section_coord(coord.floor() as i32)`.
/// See [`coord_section_from_block`].
#[inline]
pub fn coord_section_from_block_f64(coord: f64) -> i32 {
    coord_section_from_block(coord.floor() as i32)
}

/// Converts the given chunk section coordinate to the world coordinate system.
/// The returned coordinate will always be at the origin of the chunk section in world space.
///
/// This function is equivalent to `section_coord << 4`.
#[inline]
#[doc(alias = "block_coord")]
pub const fn coord_block_from_section(section_coord: i32) -> i32 {
    section_coord << 4
}
