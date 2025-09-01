use std::sync::{Arc, OnceLock};

use glam::{BVec3, DVec3, UVec3};
use voxel_math::{BBox, direction::Axis};

use crate::{
    Array, ErasedList, F64_TOLERANCE, ListEraser, MAX_SHAPE_RESOLUTION, RawVoxelShape, Simple,
    Slice, VoxelSet, list::*,
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

/// Combine two slices, returning a new slice that is unioned.
///
/// See [`combine_with`] for customized voxel merging logic.
#[inline]
#[doc(alias = "union")]
pub fn combine<'a>(lhs: &Arc<Slice<'a>>, rhs: &Arc<Slice<'a>>) -> Arc<Slice<'a>> {
    combine_with(lhs, rhs, |a, b| a || b)
}

/// Combine two slices using a function describing how to combine two single voxels.
///
/// # Panics
///
/// Panics if the given function returns `true` for two `false` voxel inputs.
pub fn combine_with<'a, F>(lhs: &Arc<Slice<'a>>, rhs: &Arc<Slice<'a>>, f: F) -> Arc<Slice<'a>>
where
    F: Fn(bool, bool) -> bool,
{
    assert!(
        !f(false, false),
        "combination function should not return true for all false inputs"
    );

    // identical slices
    if std::ptr::eq(lhs, rhs) {
        return if f(true, true) { lhs } else { empty() }.clone();
    }

    // tolerance of single-sided existence
    let lhs_tol = f(true, false);
    let rhs_tol = f(false, true);

    // singular-empty cases
    if lhs.is_empty() {
        return if rhs_tol { rhs } else { empty() }.clone();
    } else if rhs.is_empty() {
        return if lhs_tol { lhs } else { empty() }.clone();
    }

    let pair_x = list_pair(
        1,
        lhs.0.__point_pos_list_arc(Axis::X),
        rhs.0.__point_pos_list_arc(Axis::X),
        lhs_tol,
        rhs_tol,
    );
    let pair_y = list_pair(
        pair_x.__erased_len(),
        lhs.0.__point_pos_list_arc(Axis::Y),
        rhs.0.__point_pos_list_arc(Axis::Y),
        lhs_tol,
        rhs_tol,
    );
    let pair_z = list_pair(
        (pair_x.__erased_len() - 1) * (pair_y.__erased_len() - 1),
        lhs.0.__point_pos_list_arc(Axis::Z),
        rhs.0.__point_pos_list_arc(Axis::Z),
        lhs_tol,
        rhs_tol,
    );

    let set = VoxelSet::combine_with(
        &lhs.0.__as_raw().voxels,
        &rhs.0.__as_raw().voxels,
        [&*pair_x, &*pair_y, &*pair_z],
        f,
    );
    let raw = RawVoxelShape::from_arc(set.into_boxed_slice().into());

    if pair_x.__downcast_fractional_pair_double_list().is_some()
        && pair_y.__downcast_fractional_pair_double_list().is_some()
        && pair_z.__downcast_fractional_pair_double_list().is_some()
    {
        Simple(raw).into_boxed_slice()
    } else {
        #[inline]
        fn conv(x: Box<dyn PairErasedList<f64>>) -> Box<dyn ErasedList<f64>> {
            x
        }

        Array {
            raw,
            xp: conv(pair_x).into(),
            yp: conv(pair_y).into(),
            zp: conv(pair_z).into(),
        }
        .into_boxed_slice()
    }
    .into()
}

fn list_pair(
    len: usize,
    lhs: Arc<dyn ErasedList<f64>>,
    rhs: Arc<dyn ErasedList<f64>>,
    lhs_tol: bool,
    rhs_tol: bool,
) -> Box<dyn PairErasedList<f64>> {
    let lhs_lr = lhs.__erased_len() - 1;
    let rhs_lr = rhs.__erased_len() - 1;

    if lhs.__downcast_fractional_double_list().is_some()
        && rhs.__downcast_fractional_double_list().is_some()
        && len as u64 * math::int::lcm(lhs_lr as u64, rhs_lr as u64) < 256
    {
        return Box::new(FractionalPairDoubleList::new(lhs_lr, rhs_lr));
    }

    if lhs.__erased_index(lhs_lr) < rhs.__erased_index(0) - F64_TOLERANCE {
        Box::new(ChainedPairList {
            left: ListDeref(lhs),
            right: ListDeref(rhs),
            inverted: false,
        })
    } else if rhs.__erased_index(rhs_lr) < lhs.__erased_index(0) - F64_TOLERANCE {
        Box::new(ChainedPairList {
            left: ListDeref(rhs),
            right: ListDeref(lhs),
            inverted: true,
        })
    } else if Arc::ptr_eq(&lhs, &rhs) {
        Box::new(IdentityPairList(ListDeref(lhs)))
    } else {
        Box::new(SimplePairDoubleList::new(&*lhs, &*rhs, lhs_tol, rhs_tol))
    }
}
