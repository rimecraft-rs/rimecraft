use std::sync::{Arc, OnceLock};

use glam::{BVec3, DVec3, UVec3};
use voxel_math::BBox;

use crate::{
    Array, DOUBLE_BOUNDARY, ErasedList, ListEraser, RawVoxelShape, Simple, Slice, VoxelSet,
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

/// Constructs a cuboid shape within given bounds.
pub fn cuboid(bounds: BBox) -> Arc<Slice<'static>> {
    let min = bounds.min();
    let max = bounds.max();

    let fallback = || -> Arc<Slice<'static>> {
        static FALLBACK: OnceLock<Arc<Slice<'static>>> = OnceLock::new();
        FALLBACK
            .get_or_init(|| {
                Array {
                    raw: full_cube().0.__as_raw().clone(),
                    xp: Arc::new(ListEraser([min.x, max.x])),
                    yp: Arc::new(ListEraser([min.y, max.y])),
                    zp: Arc::new(ListEraser([min.z, max.z])),
                }
                .into_boxed_slice()
                .into()
            })
            .clone()
    };

    if (min - max).cmplt(DVec3::splat(DOUBLE_BOUNDARY)).any() {
        empty().clone()
    } else if min.cmplt(DVec3::splat(-DOUBLE_BOUNDARY)).any()
        || max.cmpgt(DVec3::splat(1.0000001f64)).any()
    {
        fallback()
    } else {
        let mut assigned = BVec3::FALSE;
        let mut result = UVec3::ZERO;
        for i in 0..=3u32 {
            let j = (1u32 << i) as f64;
            let d = min * j;
            let e = max * j;
            let v_precision = DVec3::splat(DOUBLE_BOUNDARY * j);
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
