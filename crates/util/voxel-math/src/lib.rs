//! Voxel math library.

mod block_pos;
mod chunk_pos;
mod chunk_section_pos;

use glam::{DVec3, IVec3};
use std::ops::ControlFlow;

pub use block_pos::BlockPos;
pub use chunk_pos::ChunkPos;
pub use chunk_section_pos::ChunkSectionPos;

mod bbox;
pub mod direction;

pub use bbox::BBox;

pub use glam;

const F64_TOLERANCE: f64 = 1.0e-7f64;

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

/// Raycasts block positions from start to end, using the given function.
pub fn raycast_f64<F, U>(start: DVec3, end: DVec3, mut f: F) -> Option<U>
where
    F: FnMut(BlockPos) -> ControlFlow<U>,
{
    if start == end {
        return None;
    }

    // small extension to endpoints
    let (start, end) = (
        start.lerp(end, -F64_TOLERANCE),
        end.lerp(start, -F64_TOLERANCE),
    );

    let mut ptr = start.floor().as_ivec3();
    // first hit
    if let ControlFlow::Break(val) = f(ptr.into()) {
        return Some(val);
    }

    let diff = end - start;
    let signs = diff.signum().as_ivec3();
    // step * iteration count = 1.0
    let step = DVec3::select(diff.cmpeq(DVec3::ZERO), DVec3::MAX, diff.signum() / diff);
    let start_fract = start.fract();
    // progress signum from zero to one
    let mut progress = step
        * DVec3::select(
            signs.cmpeq(IVec3::ONE),
            DVec3::ONE - start_fract,
            start_fract,
        );

    // consequental iteration
    while progress.cmple(DVec3::ONE).any() {
        let mpos = progress.min_position();
        ptr[mpos] += signs[mpos];
        progress[mpos] += step[mpos];

        if let ControlFlow::Break(val) = f(ptr.into()) {
            return Some(val);
        }
    }

    None
}
