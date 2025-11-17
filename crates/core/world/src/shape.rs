use std::sync::Arc;

use bitvec::BitArr;
use rimecraft_block::ProvideBlockStateExtTy;
use rimecraft_voxel_math::direction::Direction;
use voxel_shape::VoxelShapeSlice;

pub(crate) struct ShapeCache<'a> {
    collision_shape: Arc<VoxelShapeSlice<'a>>,
    solid_sides: BitArr!(for Direction::COUNT, in u8),

    exceeds_cube: bool,
    full_cube: bool,
}

impl<'a> ShapeCache<'a> {
    pub(crate) fn new<Cx>(settings: rimecraft_block::Settings<'_, Cx>) -> Self
    where
        Cx: ProvideBlockStateExtTy,
    {
        todo!()
    }
}
