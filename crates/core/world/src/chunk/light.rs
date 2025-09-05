//! Chunk lighting related stuffs.

use rimecraft_block::BlockState;
use rimecraft_packed_int_array::PackedIntArray;
use rimecraft_voxel_math::{BlockPos, coord_block_from_section, direction::Direction};

use crate::{
    chunk::{BaseChunk, BaseChunkAccess, ChunkCx, section::ComputeIndex},
    view::HeightLimit,
};

/// Bytes stores the maximum sky light that reaches each block, regardless of current time.
#[derive(Debug)]
pub struct ChunkSkyLight {
    pal: PackedIntArray,
    min_y: i32,
}

impl ChunkSkyLight {
    /// Creates a new chunk sky light.
    #[allow(clippy::missing_panics_doc)]
    pub fn new(height: HeightLimit) -> Self {
        let min_y = height.bottom() - 1;
        let j = usize::BITS - ((height.top() - min_y + 1) as usize).leading_zeros();
        Self {
            pal: PackedIntArray::from_packed(j, 256, None).unwrap(),
            min_y,
        }
    }

    fn fill(&mut self, y: i32) {
        let i = y - self.min_y;
        debug_assert!(i >= 0, "y out of range");

        for j in 0..self.pal.len() {
            self.pal.set(j, i as u32);
        }
    }
}

impl<'w, Cx> BaseChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>,
{
    pub(in crate::chunk) fn __sky_light_refresh_surface_y(mut this: impl BaseChunkAccess<'w, Cx>) {
        let i = this
            .reclaim()
            .iter_read_chunk_sections()
            .into_iter()
            .rposition(|e| !e.is_empty());
        if let Some(i) = i {
            todo!()
        } else {
            let mut skl = this.reclaim().write_chunk_sky_light();
            let min_y = skl.min_y;
            skl.fill(min_y);
        }
    }

    fn __calculate_surface_y(
        mut this: impl BaseChunkAccess<'w, Cx>,
        top_section_index: u32,
        local_x: i32,
        local_z: i32,
        min_y: i32,
    ) -> i32 {
        let height_limit = this.bca_as_bc().height_limit;
        let local_y = coord_block_from_section(
            height_limit.section_index_to_coord(top_section_index as i32) + 1,
        );
        let mut pos = BlockPos::new(local_x, local_y, local_z);
        let mut pos2 = pos + Direction::Down.offset();
        let mut bs: Option<BlockState<'w, Cx>> = None;

        for j in (0..=top_section_index).rev() {
            if let Some(section) = this
                .reclaim()
                .read_chunk_section(j as usize)
                .filter(|s| !s.is_empty())
            {
                // why 15?
                for k in (0..=15).rev() {
                    let bs2 = section.block_state(local_x as u32, k, local_z as u32);
                    todo!()
                }
            } else {
                bs = None;
                pos.0.y = coord_block_from_section(height_limit.section_index_to_coord(j as i32));
                pos2.0.y = pos.y() - 1;
            }
        }

        min_y
    }
}
