//! Obtains shape of a block.

use voxel_shape::VoxelShapeSlice;

pub trait ShapeContext<'a> {
    fn is_descending(&self) -> bool;

    fn is_above(&self, shape: &VoxelShapeSlice<'a>) -> bool;
}
