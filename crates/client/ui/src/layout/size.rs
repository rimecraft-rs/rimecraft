//! Layout sizing.

use rimecraft_render_math::screen::ScreenSize;

use crate::layout::{LayoutPack, LayoutValue};

/// The size constraint on a single axis.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum SizeConstraint {
    /// No constraints on size. The size will be determined by the engine.
    #[default]
    Free,
    /// A fixed size.
    Fixed(LayoutValue),
    /// A flexible size with optional minimum and maximum bounds.
    ///
    /// Note: always prefer using [`SizeConstraint::Free`] if both `min` and `max` are [`None`]. However, the detailed behavior is determined by the layout engine.
    Flexible {
        /// The minimum size, if any.
        min: Option<LayoutValue>,
        /// The maximum size, if any.
        max: Option<LayoutValue>,
    },
}

/// Size constraints for both horizontal and vertical axes.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SizeConstraints<Ext> {
    /// The horizontal [`SizeConstraint`].
    pub horizontal: SizeConstraint,
    /// The vertical [`SizeConstraint`].
    pub vertical: SizeConstraint,
    /// The extension data.
    pub ext: Ext,
}

impl<Ext> SizeConstraints<Ext> {
    /// Creates a new [`SizeConstraints`] with the given horizontal and vertical constraints.
    pub fn new(horizontal: SizeConstraint, vertical: SizeConstraint, ext: Ext) -> Self {
        Self {
            horizontal,
            vertical,
            ext,
        }
    }

    /// Creates a new [`SizeConstraints`] with free constraints.
    pub fn free(ext: Ext) -> Self {
        Self {
            horizontal: SizeConstraint::Free,
            vertical: SizeConstraint::Free,
            ext,
        }
    }

    /// Creates a new [`SizeConstraints`] with fixed constraints.
    pub fn fixed(horizontal: LayoutValue, vertical: LayoutValue, ext: Ext) -> Self {
        Self {
            horizontal: SizeConstraint::Fixed(horizontal),
            vertical: SizeConstraint::Fixed(vertical),
            ext,
        }
    }

    /// Creates a new [`SizeConstraints`] from a [`ScreenSize`], with fixed constraints.
    pub fn from_size(size: ScreenSize, ext: Ext) -> Self {
        Self {
            horizontal: SizeConstraint::Fixed(LayoutValue::Px(size.horizontal)),
            vertical: SizeConstraint::Fixed(LayoutValue::Px(size.vertical)),
            ext,
        }
    }
}

impl<Ext> SizeConstraints<Ext> {
    /// Returns the minimum size available.
    pub fn minimum_size(&self) -> LayoutPack<Option<LayoutValue>> {
        let horizontal = match self.horizontal {
            SizeConstraint::Free => Some(LayoutValue::Px(0.0)),
            SizeConstraint::Fixed(h) => Some(h),
            SizeConstraint::Flexible { min: Some(m), .. } => Some(m),
            SizeConstraint::Flexible { min: None, .. } => Some(LayoutValue::Px(0.0)),
        };
        let vertical = match self.vertical {
            SizeConstraint::Free => Some(LayoutValue::Px(0.0)),
            SizeConstraint::Fixed(v) => Some(v),
            SizeConstraint::Flexible { min: Some(m), .. } => Some(m),
            SizeConstraint::Flexible { min: None, .. } => Some(LayoutValue::Px(0.0)),
        };
        LayoutPack::new(horizontal, vertical)
    }

    /// Returns the maximum size available.
    pub fn maximum_size(&self) -> LayoutPack<Option<LayoutValue>> {
        let horizontal = match self.horizontal {
            SizeConstraint::Free => None,
            SizeConstraint::Fixed(h) => Some(h),
            SizeConstraint::Flexible { max: Some(m), .. } => Some(m),
            SizeConstraint::Flexible { max: None, .. } => None,
        };
        let vertical = match self.vertical {
            SizeConstraint::Free => None,
            SizeConstraint::Fixed(v) => Some(v),
            SizeConstraint::Flexible { max: Some(m), .. } => Some(m),
            SizeConstraint::Flexible { max: None, .. } => None,
        };
        LayoutPack::new(horizontal, vertical)
    }
}
