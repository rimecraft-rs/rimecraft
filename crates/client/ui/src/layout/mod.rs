//! Layout constraints and measurement utilities.

use std::{fmt::Debug, ops::Not};

pub mod engine;
pub mod position;
pub mod size;

/// The component to refer to for layout calculations. [`LayoutReference::Upstream`] should refer to the nearest parent element that can provide layout context, while [`LayoutReference::Root`] should refer to the screen in most cases.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LayoutReference {
    /// The root component, usually the screen.
    Root,
    /// The nearest upstream component that is suitable for providing layout context, usually the parent element.
    Upstream,
    /// The component itself.
    #[default]
    This,
}

/// Measurements used during layout calculations.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct LayoutMeasurements {
    /// The measured value of the root component.
    pub root: f32,
    /// The measured value of the upstream component.
    pub upstream: f32,
    /// The measured value of the current component, if known.
    pub this: Option<f32>,
}

impl LayoutMeasurements {
    /// Creates a new [`LayoutMeasurements`].
    pub fn new(root: f32, upstream: f32, this: Option<f32>) -> Self {
        Self {
            root,
            upstream,
            this,
        }
    }
}

impl From<(f32, f32, Option<f32>)> for LayoutMeasurements {
    fn from((root, upstream, this): (f32, f32, Option<f32>)) -> Self {
        Self::new(root, upstream, this)
    }
}

/// A layout value, either in pixels or as a percentage of a [`LayoutReference`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum LayoutValue {
    /// Absolute value in pixels.
    Px(f32),
    /// Relative value as a percentage of a [`LayoutReference`].
    Percent(f32, LayoutReference),
}

impl LayoutValue {
    /// Creates a new [`LayoutValue`] in pixels.
    pub fn px(value: f32) -> Self {
        Self::Px(value)
    }

    /// Creates a new [`LayoutValue`] as a percentage of a [`LayoutReference`].
    pub fn percent(value: f32, reference: LayoutReference) -> Self {
        Self::Percent(value, reference)
    }

    /// Resolves the [`LayoutValue`] to an absolute pixel value based on the given reference size.
    pub fn resolve(&self, measurements: LayoutMeasurements) -> f32 {
        match self {
            Self::Px(value) => *value,
            Self::Percent(value, reference) => match reference {
                LayoutReference::Root => measurements.root * (*value / 100.0),
                LayoutReference::Upstream => measurements.upstream * (*value / 100.0),
                LayoutReference::This => {
                    measurements.this.unwrap_or(measurements.upstream) * (*value / 100.0)
                }
            },
        }
    }
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

impl<V> LayoutWithCenter<V>
where
    V: Debug + Clone + PartialEq,
{
    /// Maps the inner value using the given function, preserving the [`LayoutWithCenter::Center`] variant.
    pub fn map<F, U>(&self, f: F) -> LayoutWithCenter<U>
    where
        F: FnOnce(&V) -> U,
        U: Debug + Clone + PartialEq,
    {
        match self {
            Self::Center => LayoutWithCenter::Center,
            Self::Value(v) => LayoutWithCenter::Value(f(v)),
        }
    }
}

/// The horizontal edge for layout anchoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutHorizontalEdge {
    /// The leading side.
    Leading,
    /// The trailing side.
    Trailing,
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

/// A generic edge for layout anchoring, can represent either horizontal or vertical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutGenericEdge {
    /// The start side. Equivalent to [`Leading`](LayoutHorizontalEdge::Leading) for horizontal and [`Top`](LayoutVerticalEdge::Top) for vertical.
    Start,
    /// The end side. Equivalent to [`Trailing`](LayoutHorizontalEdge::Trailing) for horizontal and [`Bottom`](LayoutVerticalEdge::Bottom) for vertical.
    End,
}

impl From<LayoutGenericEdge> for LayoutHorizontalEdge {
    fn from(edge: LayoutGenericEdge) -> Self {
        match edge {
            LayoutGenericEdge::Start => Self::Leading,
            LayoutGenericEdge::End => Self::Trailing,
        }
    }
}

impl From<LayoutGenericEdge> for LayoutVerticalEdge {
    fn from(edge: LayoutGenericEdge) -> Self {
        match edge {
            LayoutGenericEdge::Start => Self::Top,
            LayoutGenericEdge::End => Self::Bottom,
        }
    }
}

impl From<LayoutHorizontalEdge> for LayoutGenericEdge {
    fn from(edge: LayoutHorizontalEdge) -> Self {
        match edge {
            LayoutHorizontalEdge::Leading => Self::Start,
            LayoutHorizontalEdge::Trailing => Self::End,
        }
    }
}

impl From<LayoutVerticalEdge> for LayoutGenericEdge {
    fn from(edge: LayoutVerticalEdge) -> Self {
        match edge {
            LayoutVerticalEdge::Top => Self::Start,
            LayoutVerticalEdge::Bottom => Self::End,
        }
    }
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
            LayoutWithCenter::Value(LayoutHorizontalEdge::Leading),
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
