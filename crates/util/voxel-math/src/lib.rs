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

pub use glam::{DVec3, IVec3};
