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

    let pair_x = fast_list_pair(
        1,
        ListDeref(lhs.0.__point_pos_list_arc(Axis::X)),
        ListDeref(rhs.0.__point_pos_list_arc(Axis::X)),
        lhs_tol,
        rhs_tol,
    );
    let pair_y = fast_list_pair(
        pair_x.__erased_len(),
        ListDeref(lhs.0.__point_pos_list_arc(Axis::Y)),
        ListDeref(rhs.0.__point_pos_list_arc(Axis::Y)),
        lhs_tol,
        rhs_tol,
    );
    let pair_z = fast_list_pair(
        (pair_x.__erased_len() - 1) * (pair_y.__erased_len() - 1),
        ListDeref(lhs.0.__point_pos_list_arc(Axis::Z)),
        ListDeref(rhs.0.__point_pos_list_arc(Axis::Z)),
        lhs_tol,
        rhs_tol,
    );

    let set = VoxelSet::combine_with(
        &lhs.0.__as_raw().voxels,
        &rhs.0.__as_raw().voxels,
        [&pair_x, &pair_y, &pair_z],
        f,
    );
    let raw = RawVoxelShape::from_arc(set.into_boxed_slice().into());

    if pair_x.__downcast_fractional_pair_double_list().is_some()
        && pair_y.__downcast_fractional_pair_double_list().is_some()
        && pair_z.__downcast_fractional_pair_double_list().is_some()
    {
        Simple(raw).into_boxed_slice()
    } else {
        Array {
            raw,
            xp: pair_x.into_boxed_list().into(),
            yp: pair_y.into_boxed_list().into(),
            zp: pair_z.into_boxed_list().into(),
        }
        .into_boxed_slice()
    }
    .into()
}

/// Tests if the two slices match **anywhere** using the given function.
///
/// # Panics
///
/// Panics if the given function returns `true` for two `false` voxel inputs.
pub fn matches_anywhere<'a, F>(lhs: &Arc<Slice<'a>>, rhs: &Arc<Slice<'a>>, f: F) -> bool
where
    F: Fn(bool, bool) -> bool,
{
    assert!(
        !f(false, false),
        "matching function should not return true for all false inputs"
    );

    let l_empty = lhs.is_empty();
    let r_empty = rhs.is_empty();

    // (singular)empty cases
    if l_empty || r_empty {
        return f(!l_empty, !r_empty);
    }

    // identical slices
    if Arc::ptr_eq(lhs, rhs) {
        return f(true, true);
    }

    let ltrf = f(true, false);
    let lfrt = f(false, true);

    if lhs
        .0
        .__max_vectorized()
        .cmplt(rhs.0.__min_vectorized() - DVec3::splat(F64_TOLERANCE))
        .any()
        || rhs
            .0
            .__max_vectorized()
            .cmplt(lhs.0.__min_vectorized() - DVec3::splat(F64_TOLERANCE))
            .any()
    {
        return ltrf || lfrt;
    }

    let p1 = fast_list_pair(
        1,
        ListDeref(lhs.0.__point_pos_list_arc(Axis::X)),
        ListDeref(rhs.0.__point_pos_list_arc(Axis::X)),
        ltrf,
        lfrt,
    );

    let p2 = fast_list_pair(
        p1.__erased_len() - 1,
        ListDeref(lhs.0.__point_pos_list_arc(Axis::Y)),
        ListDeref(rhs.0.__point_pos_list_arc(Axis::Y)),
        ltrf,
        lfrt,
    );

    let p3 = fast_list_pair(
        (p1.__erased_len() - 1) * (p2.__erased_len() - 1),
        ListDeref(lhs.0.__point_pos_list_arc(Axis::Z)),
        ListDeref(rhs.0.__point_pos_list_arc(Axis::Z)),
        ltrf,
        lfrt,
    );

    let l_voxels = &lhs.0.__as_raw().voxels;
    let r_voxels = &rhs.0.__as_raw().voxels;

    let mut result = false;
    p1.__peek_pair_erased_iter(&mut |iter| {
        for item1 in iter {
            p2.__peek_pair_erased_iter(&mut |iter| {
                for item2 in iter {
                    p3.__peek_pair_erased_iter(&mut |iter| {
                        for item3 in iter {
                            if f(
                                l_voxels.in_bounds_and_contains(item1.x, item2.x, item3.x),
                                r_voxels.in_bounds_and_contains(item1.y, item2.y, item3.y),
                            ) {
                                result = true;
                                break;
                            }
                        }
                    });
                    if result {
                        break;
                    }
                }
            });
            if result {
                break;
            }
        }
    });

    result
}

/// Returns true if the union of the two shapes covers the entire cube.
///
/// _The internal implementation is costly._
pub fn union_covers_full_cube<'a>(lhs: &Arc<Slice<'a>>, rhs: &Arc<Slice<'a>>) -> bool {
    if Arc::ptr_eq(lhs, full_cube()) || Arc::ptr_eq(rhs, full_cube()) {
        true
    } else if lhs.is_empty() && rhs.is_empty() {
        false
    } else {
        !matches_anywhere(full_cube(), &combine(lhs, rhs), |a, _| a)
    }
}

fn fast_list_pair<T>(
    len: usize,
    lhs: T,
    rhs: T,
    lhs_tol: bool,
    rhs_tol: bool,
) -> FastDoublePairList<T>
where
    T: ErasedList<f64>,
{
    let lhs_lr = lhs.__erased_len() - 1;
    let rhs_lr = rhs.__erased_len() - 1;

    if lhs.__downcast_fractional_double_list().is_some()
        && rhs.__downcast_fractional_double_list().is_some()
        && len as u64 * math::int::lcm(lhs_lr as u64, rhs_lr as u64) < 256
    {
        return FastDoublePairList::Fractional(FractionalPairDoubleList::new(lhs_lr, rhs_lr));
    }

    if lhs.__erased_index(lhs_lr) < rhs.__erased_index(0) - F64_TOLERANCE {
        FastDoublePairList::Chained(ChainedPairList {
            left: lhs,
            right: rhs,
            inverted: false,
        })
    } else if rhs.__erased_index(rhs_lr) < lhs.__erased_index(0) - F64_TOLERANCE {
        FastDoublePairList::Chained(ChainedPairList {
            left: rhs,
            right: lhs,
            inverted: true,
        })
    } else if lhs.__is_identical(&rhs) {
        FastDoublePairList::Identity(IdentityPairList(lhs))
    } else {
        FastDoublePairList::Simple(SimplePairDoubleList::new(&lhs, &rhs, lhs_tol, rhs_tol))
    }
}
