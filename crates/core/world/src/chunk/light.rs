//! Chunk lighting related stuffs.

use std::sync::Arc;

use rimecraft_block::{BlockState, BlockStateExt};
use rimecraft_global_cx::Hold;
use rimecraft_packed_int_array::PackedIntArray;
use rimecraft_voxel_math::{BlockPos, coord_block_from_section, direction::Direction};

use crate::{
    NestedBlockStateExt,
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

    fn set(&mut self, index: usize, y: i32) {
        self.pal.set(index, (y - self.min_y) as u32);
    }
}

fn skl_packed_index(local_x: i32, local_z: i32) -> i32 {
    local_x + local_z * 16
}

impl<'w, Cx> BaseChunk<'w, Cx>
where
    Cx: ChunkCx<'w> + ComputeIndex<Cx::BlockStateList, BlockState<'w, Cx>>,
{
    pub(in crate::chunk) fn __csl_refresh_surface_y(mut this: impl BaseChunkAccess<'w, Cx>) {
        let i = this
            .reclaim()
            .iter_read_chunk_sections()
            .into_iter()
            .rposition(|e| !e.is_empty());

        //SAFETY: the upcoming OPs involving `this` are all unassociated with chunk sky light
        //NECESSITY: or we may have 16 * 16 locking operations if not lock free.
        let mut skl = unsafe { &mut *std::ptr::from_mut(&mut this) }
            .reclaim()
            .write_chunk_sky_light();

        let min_y = skl.min_y;
        if let Some(i) = i {
            for j in 0..16 {
                for k in 0..16 {
                    let l = Self::__csl_calculate_surface_y(this.reclaim(), i as u32, k, j, min_y)
                        .max(min_y);
                    skl.set(skl_packed_index(k, j) as usize, l);
                }
            }
        } else {
            let min_y = skl.min_y;
            skl.fill(min_y);
        }
    }

    fn __csl_calculate_surface_y(
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
                    if bs.as_ref().map_or_else(
                        || face_blocks_light_upper_air(&bs2),
                        |bs| face_blocks_light(bs, &bs2),
                    ) {
                        return pos.y();
                    }

                    bs = Some(bs2);
                    pos = pos2;
                    pos2 = pos2.mv(Direction::Down, 1);
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

/// Returns the opaque shape of given block state in the specified direction.
pub fn opaque_shape_of_state<'a, Cx>(
    state: &BlockState<'a, Cx>,
    direction: Direction,
) -> &'a Arc<voxel_shape::Slice<'a>>
where
    Cx: ChunkCx<'a>,
{
    if is_trivial_for_lighting(state) {
        voxel_shape::empty()
    } else {
        let nested: &NestedBlockStateExt<'_> = state.data().get_held();
        nested.culling_face(direction)
    }
}

#[inline]
pub(crate) fn is_trivial_for_lighting<'a, Cx>(state: &BlockState<'a, Cx>) -> bool
where
    Cx: ChunkCx<'a>,
{
    !state.settings().opaque || !state.has_sided_transparency()
}

fn face_blocks_light<'a, Cx>(upper: &BlockState<'a, Cx>, lower: &BlockState<'a, Cx>) -> bool
where
    Cx: ChunkCx<'a>,
{
    if lower.data().opacity() == 0 {
        let shape_up = opaque_shape_of_state(upper, Direction::Down);
        let shape_down = opaque_shape_of_state(lower, Direction::Up);
        voxel_shape::union_covers_full_cube(shape_up, shape_down)
    } else {
        true
    }
}

fn face_blocks_light_upper_air<'a, Cx>(lower: &BlockState<'a, Cx>) -> bool
where
    Cx: ChunkCx<'a>,
{
    if lower.data().opacity() == 0 {
        let shape_up = voxel_shape::empty();
        let shape_down = opaque_shape_of_state(lower, Direction::Up);
        voxel_shape::union_covers_full_cube(shape_up, shape_down)
    } else {
        true
    }
}
