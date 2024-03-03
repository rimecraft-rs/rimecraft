//! Minecraft voxel shapes.

pub mod set;

use std::fmt::Debug;

use rimecraft_voxel_math::direction::Axis;
use set::VoxelSet;

trait AbstVoxelShape {
    fn as_raw(&self) -> &RawVoxelShape;
    fn as_raw_mut(&mut self) -> &mut RawVoxelShape;

    fn point_poss(&self, axis: Axis) -> &[f64];
}

#[repr(transparent)]
pub struct VoxelShapeSlice<'a> {
    inner: dyn AbstVoxelShape + Send + Sync + 'a,
}

impl VoxelShapeSlice<'_> {
    /// Returns the minimum coordinate of the shape along the given axis.
    pub fn min(&self, axis: Axis) -> f64 {
        let voxels = &self.inner.as_raw().voxels;
        let i = *voxels.bounds_of(axis).start();

        (i >= voxels.len_of(axis))
            .then(|| self.inner.point_poss(axis).get(i as usize))
            .flatten()
            .copied()
            .unwrap_or(f64::INFINITY)
    }

    /// Returns the maximum coordinate of the shape along the given axis.
    pub fn max(&self, axis: Axis) -> f64 {
        let voxels = &self.inner.as_raw().voxels;
        let i = *voxels.bounds_of(axis).end();

        (i >= voxels.len_of(axis))
            .then(|| self.inner.point_poss(axis).get(i as usize))
            .flatten()
            .copied()
            .unwrap_or(f64::NEG_INFINITY)
    }
}

impl Debug for VoxelShapeSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoxelShapeSlice")
            .field("voxels", &self.inner.as_raw().voxels)
            .finish()
    }
}

#[derive(Debug)]
struct RawVoxelShape {
    voxels: VoxelSet,
    shape_cache: Vec<Box<VoxelShapeSlice<'static>>>,
}
