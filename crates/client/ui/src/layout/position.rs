use crate::layout::{LayoutPack, LayoutPivot};

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum PositionConstraint {
    Absolute { x: f32, y: f32 },
    Relative { x: f32, y: f32 },
}

impl PositionConstraint {
    /// Creates a new absolute position constraint.
    pub fn absolute(x: f32, y: f32) -> Self {
        Self::Absolute { x, y }
    }

    /// Creates a new relative position constraint.
    pub fn relative(x: f32, y: f32) -> Self {
        Self::Relative { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionConstraints {
    pub parent_pivot: LayoutPivot,
    pub child_pivot: LayoutPivot,
    pub offset: LayoutPack<PositionConstraint>,
}
