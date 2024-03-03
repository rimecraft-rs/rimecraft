//! Minecraft voxel shapes.

pub mod set;

use std::fmt::Debug;

use set::VoxelSet;

trait AbstVoxelShape {}

#[repr(transparent)]
pub struct VoxelShapeSlice<'a> {
    inner: dyn AbstVoxelShape + Send + Sync + 'a,
}

impl Debug for VoxelShapeSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug)]
struct RawVoxelShape {
    voxels: VoxelSet,
    shape_cache: Vec<Box<VoxelShapeSlice<'static>>>,
}
