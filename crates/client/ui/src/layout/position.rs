//! Layout positioning.

use crate::layout::{LayoutAnchor, LayoutPack, LayoutReference, LayoutValue};

/// The position constraint on a single axis.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub struct PositionConstraint(pub LayoutValue, pub LayoutReference);

impl PositionConstraint {
    /// Creates a new [`PositionConstraint`].
    pub fn new(value: LayoutValue, reference: LayoutReference) -> Self {
        Self(value, reference)
    }
}

impl From<(LayoutValue, LayoutReference)> for PositionConstraint {
    fn from((value, reference): (LayoutValue, LayoutReference)) -> Self {
        Self::new(value, reference)
    }
}

/// Position constraints on both horizontal and vertical axes, with layout pivots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionConstraints<Ext> {
    /// The anchor point on this component.
    pub this_anchor: LayoutAnchor,
    /// The anchor point on the target component, often the parent element.
    pub target_anchor: LayoutAnchor,
    /// The offset from `target_anchor` to `this_anchor`.
    pub offset: LayoutPack<PositionConstraint>,
    /// The extension data.
    pub ext: Ext,
}

impl<Ext> PositionConstraints<Ext> {
    /// Creates a new [`PositionConstraints`].
    pub fn new(
        this_anchor: LayoutAnchor,
        target_anchor: LayoutAnchor,
        offset: LayoutPack<PositionConstraint>,
        ext: Ext,
    ) -> Self {
        Self {
            target_anchor,
            this_anchor,
            offset,
            ext,
        }
    }

    /// Creates a new [`PositionConstraints`] with both anchors at the origin (top-left).
    pub fn origin_aligned(offset: LayoutPack<PositionConstraint>, ext: Ext) -> Self {
        Self {
            target_anchor: LayoutAnchor::origin(),
            this_anchor: LayoutAnchor::origin(),
            offset,
            ext,
        }
    }

    /// Creates a new [`PositionConstraints`] with both anchors at the center.
    pub fn center_aligned(offset: LayoutPack<PositionConstraint>, ext: Ext) -> Self {
        Self {
            target_anchor: LayoutAnchor::center(),
            this_anchor: LayoutAnchor::center(),
            offset,
            ext,
        }
    }
}
