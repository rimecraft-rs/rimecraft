//! Layout engine for computing layout properties.

use rimecraft_local_cx::LocalContext;
use rimecraft_render_math::screen::{ScreenPos, ScreenSize};

use crate::{
    ProvideUiTy,
    layout::{
        LayoutGenericEdge, LayoutMeasurements, LayoutReference, LayoutWithCenter,
        pos::{PosConstraint, PosConstraints},
        size::{SizeConstraint, SizeConstraints},
    },
};

/// A layout engine that resolves layout constraints into actual sizes and positions.
pub trait LayoutEngine<Cx>
where
    Cx: ProvideUiTy,
{
    /// Resolves size constraints into actual sizes, relative to the screen coordinate system.
    fn resolve_size_constraints<Local>(
        &self,
        cx: Local,
        upstream_size: ScreenSize,
        constraints: &SizeConstraints<Cx::SizeConstraintsExt>,
    ) -> ScreenSize
    where
        Local: LocalContext<ScreenSize>;

    /// Resolves position constraints into actual positions, relative to the screen coordinate system.
    fn resolve_position_constraints<Local>(
        &self,
        cx: Local,
        upstream_pos: ScreenPos,
        upstream_size: ScreenSize,
        element_size: ScreenSize,
        constraints: &PosConstraints<Cx::PosConstraintsExt>,
    ) -> ScreenPos
    where
        Local: LocalContext<ScreenSize>;
}

/// The default layout engine implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefaultLayoutEngine<Cx>
where
    Cx: ProvideUiTy,
    Cx::LayoutMeasurementsExt: Default,
{
    _marker: std::marker::PhantomData<Cx>,
}

impl<Cx> LayoutEngine<Cx> for DefaultLayoutEngine<Cx>
where
    Cx: ProvideUiTy,
{
    fn resolve_size_constraints<Local>(
        &self,
        cx: Local,
        upstream_size: ScreenSize,
        constraints: &SizeConstraints<Cx::SizeConstraintsExt>,
    ) -> ScreenSize
    where
        Local: LocalContext<ScreenSize>,
    {
        let screen_size = LocalContext::<ScreenSize>::acquire(cx);

        fn resolve<Ext>(constraint: &SizeConstraint, upstream: f32, screen: f32) -> f32
        where
            Ext: Default + Copy,
        {
            let measurements = LayoutMeasurements {
                root: screen,
                upstream,
                this: None,
                ext: Ext::default(),
            };
            match constraint {
                SizeConstraint::Free => upstream,
                SizeConstraint::Fixed(value) => value.resolve(measurements),
                SizeConstraint::Flexible { min, max } => {
                    let mut size = upstream;
                    if let Some(min) = min {
                        let min = min.resolve(measurements);
                        if size < min {
                            size = min;
                        }
                    }
                    if let Some(max) = max {
                        let max = max.resolve(measurements);
                        if size > max {
                            size = max;
                        }
                    }
                    size
                }
            }
        }

        let horizontal = resolve::<Cx::LayoutMeasurementsExt>(
            &constraints.horizontal,
            upstream_size.horizontal,
            screen_size.horizontal,
        );
        let vertical = resolve::<Cx::LayoutMeasurementsExt>(
            &constraints.vertical,
            upstream_size.vertical,
            screen_size.vertical,
        );
        ScreenSize::new(horizontal, vertical)
    }

    fn resolve_position_constraints<Local>(
        &self,
        cx: Local,
        upstream_pos: ScreenPos,
        upstream_size: ScreenSize,
        element_size: ScreenSize,
        constraints: &PosConstraints<Cx::PosConstraintsExt>,
    ) -> ScreenPos
    where
        Local: LocalContext<ScreenSize>,
    {
        let screen_size = LocalContext::<ScreenSize>::acquire(cx);

        fn resolve<Ext>(
            constraint: &PosConstraint,
            this_edge: LayoutWithCenter<LayoutGenericEdge>,
            target_edge: LayoutWithCenter<LayoutGenericEdge>,
            upstream_pos: f32,
            upstream_size: f32,
            element_size: f32,
            screen_size: f32,
        ) -> f32
        where
            Ext: Default + Copy,
        {
            let measurements = LayoutMeasurements {
                root: screen_size,
                upstream: upstream_size,
                this: Some(element_size),
                ext: Ext::default(),
            };
            let pixels = constraint.0.resolve(measurements);
            let baseline = match constraint.1 {
                LayoutReference::Root => match target_edge {
                    LayoutWithCenter::Center => screen_size / 2.0,
                    LayoutWithCenter::Value(LayoutGenericEdge::Start) => 0.0,
                    LayoutWithCenter::Value(LayoutGenericEdge::End) => screen_size,
                },
                LayoutReference::Upstream | LayoutReference::This => {
                    upstream_pos
                        + match target_edge {
                            LayoutWithCenter::Center => upstream_size / 2.0,
                            LayoutWithCenter::Value(LayoutGenericEdge::Start) => 0.0,
                            LayoutWithCenter::Value(LayoutGenericEdge::End) => upstream_size,
                        }
                }
            };
            let this_offset = match this_edge {
                LayoutWithCenter::Center => element_size / 2.0,
                LayoutWithCenter::Value(LayoutGenericEdge::Start) => 0.0,
                LayoutWithCenter::Value(LayoutGenericEdge::End) => element_size,
            };
            baseline + pixels - this_offset
        }

        let horizontal = resolve::<Cx::LayoutMeasurementsExt>(
            &constraints.offset.horizontal,
            constraints.this_anchor.horizontal.map(|e| (*e).into()),
            constraints.target_anchor.horizontal.map(|e| (*e).into()),
            upstream_pos.horizontal,
            upstream_size.horizontal,
            element_size.horizontal,
            screen_size.horizontal,
        );
        let vertical = resolve::<Cx::LayoutMeasurementsExt>(
            &constraints.offset.vertical,
            constraints.this_anchor.vertical.map(|e| (*e).into()),
            constraints.target_anchor.vertical.map(|e| (*e).into()),
            upstream_pos.vertical,
            upstream_size.vertical,
            element_size.vertical,
            screen_size.vertical,
        );
        ScreenPos::new(horizontal, vertical)
    }
}
