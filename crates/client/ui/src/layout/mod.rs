//! Layout constraints and measurement utilities.

use std::{fmt::Debug, ops::Not};

pub mod engine;
pub mod position;
pub mod size;

/// A layout value, either in pixels or as a percentage of a [`LayoutReference`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum LayoutValue {
    /// Absolute value in pixels.
    Px(f32),
    /// Relative value as a percentage of a [`LayoutReference`].
    Percent(f32, Option<LayoutReference>),
}

/// The component to refer to for layout calculations. [`LayoutReference::Upstream`] should refer to the nearest parent element that can provide layout context, while [`LayoutReference::Root`] should refer to the screen in most cases. If referring to self is required, consider wrapping this in an [`Option`] and using [`None`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LayoutReference {
    /// The root component, usually the screen.
    Root,
    /// The nearest upstream component that is suitable for providing layout context, usually the parent element.
    #[default]
    Upstream,
}

/// The axis for layout operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutAxis {
    /// The horizontal axis.
    Horizontal,
    /// The vertical axis.
    Vertical,
}

impl LayoutAxis {
    /// Returns the orthogonal layout axis.
    pub fn orthogonal(&self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

impl Not for LayoutAxis {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.orthogonal()
    }
}

/// The direction for layout operations.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutDirection {
    /// The start direction.
    #[default]
    Start,
    /// The end direction.
    End,
}

impl LayoutDirection {
    /// Returns the opposite layout direction.
    pub fn opposite(&self) -> Self {
        match self {
            Self::Start => Self::End,
            Self::End => Self::Start,
        }
    }
}

impl Not for LayoutDirection {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.opposite()
    }
}

/// Alignment options for layout.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LayoutAlignment {
    /// Aligns to the start.
    #[default]
    Start,
    /// Aligns to the center.
    Center,
    /// Aligns to the end.
    End,
    /// Stretches to fill the available space.
    Stretch,
}

/// Justification options for layout.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LayoutJustification {
    /// Justifies to the start.
    Start,
    /// Justifies to the center.
    Center,
    /// Justifies to the end.
    End,
    /// Distributes space between items.
    #[default]
    SpaceBetween,
    /// Distributes space around items.
    SpaceAround,
    /// Distributes space evenly, including at the ends.
    SpaceEvenly,
}

/// Wraps a value to provide an option for centering.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutWithCenter<V>
where
    V: Debug + Clone + PartialEq,
{
    /// The center value.
    Center,
    /// A specific value.
    Value(V),
}

/// The horizontal edge for layout anchoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutHorizontalEdge {
    /// The left side.
    Left,
    /// The right side.
    Right,
}

/// The vertical edge for layout anchoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutVerticalEdge {
    /// The top side.
    Top,
    /// The bottom side.
    Bottom,
}

/// The edge for layout anchoring, either [`Horizontal`](LayoutHorizontalEdge) or [`Vertical`](LayoutVerticalEdge).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutEdge {
    /// The horizontal edge.
    Horizontal(LayoutHorizontalEdge),
    /// The vertical edge.
    Vertical(LayoutVerticalEdge),
}

/// An anchor point for layout positioning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutAnchor {
    /// The horizontal anchor that can be centered.
    pub horizontal: LayoutWithCenter<LayoutHorizontalEdge>,
    /// The vertical anchor that can be centered.
    pub vertical: LayoutWithCenter<LayoutVerticalEdge>,
}

impl LayoutAnchor {
    /// Creates a new [`LayoutAnchor`].
    pub fn new(
        horizontal: LayoutWithCenter<LayoutHorizontalEdge>,
        vertical: LayoutWithCenter<LayoutVerticalEdge>,
    ) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    /// Creates a new [`LayoutAnchor`] centered on both axes.
    pub fn center() -> Self {
        Self::new(LayoutWithCenter::Center, LayoutWithCenter::Center)
    }

    /// Creates a new [`LayoutAnchor`] at the origin (top-left).
    pub fn origin() -> Self {
        Self::new(
            LayoutWithCenter::Value(LayoutHorizontalEdge::Left),
            LayoutWithCenter::Value(LayoutVerticalEdge::Top),
        )
    }
}

/// Packs values for both horizontal and vertical axes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutPack<V>
where
    V: Debug + PartialEq,
{
    /// The horizontal value.
    pub horizontal: V,
    /// The vertical value.
    pub vertical: V,
}

impl<V> LayoutPack<V>
where
    V: Debug + PartialEq,
{
    /// Creates a new [`LayoutPack`].
    pub fn new(horizontal: V, vertical: V) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    /// Gets the value for the given [`LayoutAxis`].
    pub fn axis(&self, axis: LayoutAxis) -> &V {
        match axis {
            LayoutAxis::Horizontal => &self.horizontal,
            LayoutAxis::Vertical => &self.vertical,
        }
    }
}
