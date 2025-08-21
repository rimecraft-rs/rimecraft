use std::sync::{Arc, OnceLock};

use glam::{BVec3, DVec3, UVec3};
use voxel_math::BBox;

use crate::{
    Array, ErasedList, F64_TOLERANCE, ListEraser, MAX_SHAPE_RESOLUTION, RawVoxelShape, Simple,
    Slice, VoxelSet,
};

/// An empty voxel shape slice.
pub fn empty() -> &'static Arc<Slice<'static>> {
    static EMPTY: OnceLock<Arc<Slice<'static>>> = OnceLock::new();
    EMPTY.get_or_init(|| {
        let p: Arc<dyn ErasedList<f64>> = Arc::new(ListEraser([0f64]));
        Array {
            raw: RawVoxelShape::from_arc(VoxelSet::new((0, 0, 0).into()).into_boxed_slice().into()),
            xp: p.clone(),
            yp: p.clone(),
            zp: p,
        }
        .into_boxed_slice()
        .into()
    })
}

/// A full cube.
pub fn full_cube() -> &'static Arc<Slice<'static>> {
    static FULL_CUBE: OnceLock<Arc<Slice<'static>>> = OnceLock::new();
    FULL_CUBE.get_or_init(|| {
        Simple(RawVoxelShape::from_arc({
            let mut vs = VoxelSet::new((1, 1, 1).into());
            vs.set(0, 0, 0);
            vs.into_boxed_slice().into()
        }))
        .into_boxed_slice()
        .into()
    })
}

/// Builds a cuboid shape within given bounding box which use coordinates from 0 to 1 in each axis.
///
/// # Voxel Representation
///
/// Built cuboid may use bitset to store voxels internally only if three axises' bounds are aligned
/// to the resolution smaller than 1/8 of a full cube, also including 1/4, 1/2 and 1/1 who uses
/// full cube directly.
///
/// Boxes not aligned to resolutions given above are built in a discrete manner without bitset optimization.
pub fn cuboid(bounds: BBox) -> Arc<Slice<'static>> {
    let min = bounds.min();
    let max = bounds.max();

    // shape without bitset optimization
    let fallback = || -> Arc<Slice<'static>> {
        Array {
            raw: full_cube().0.__as_raw().clone(),
            xp: Arc::new(ListEraser([min.x, max.x])),
            yp: Arc::new(ListEraser([min.y, max.y])),
            zp: Arc::new(ListEraser([min.z, max.z])),
        }
        .into_boxed_slice()
        .into()
    };

    if (max - min).cmplt(DVec3::splat(F64_TOLERANCE)).any() {
        empty().clone()
    } else if min.cmplt(DVec3::splat(-F64_TOLERANCE)).any()
        || max.cmpgt(DVec3::splat(1.0000001f64)).any()
    {
        fallback()
    } else {
        let mut assigned = BVec3::FALSE;
        let mut result = UVec3::ZERO;
        for i in 0..=(MAX_SHAPE_RESOLUTION.trailing_zeros()) {
            let j = (1u32 << i) as f64; // 1, 2, 4, 8.
            let d = min * j;
            let e = max * j;
            let v_precision = DVec3::splat(F64_TOLERANCE * j);
            let bmax = (d - d.round()).abs().cmplt(v_precision);
            let bmin = (e - e.round()).abs().cmplt(v_precision);
            let bmerged = bmax & bmin;
            // if i == 0 && bmerged.all() {
            //     return full_cube().clone();
            // }
            let bdiff = bmerged & (assigned ^ bmerged);
            assigned |= bdiff;
            result = UVec3::select(bdiff, UVec3::splat(i), result);
            // if assigned.all() {
            //     break;
            // }

            // left for auto-vectorization
            // TODO: discomment if vectorization doesn't work
        }

        if assigned.all() {
            let shifted = UVec3::ONE << result;
            let rb_min = (min * shifted.as_dvec3()).round().as_uvec3();
            let rb_max = (max * shifted.as_dvec3()).round().as_uvec3();
            Simple(RawVoxelShape::from_arc(
                VoxelSet::with_bounds(
                    shifted.into(),
                    crate::set::Bounds {
                        x: rb_min.x..rb_max.x,
                        y: rb_min.y..rb_max.y,
                        z: rb_min.z..rb_max.z,
                    },
                )
                .into_boxed_slice()
                .into(),
            ))
            .into_boxed_slice()
            .into()
        } else {
            fallback()
        }
    }
}
