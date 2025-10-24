//! Layout constraints and measurement utilities.

use std::{fmt::Debug, ops::Not};

pub mod position;
pub mod size;

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum LayoutValue {
    Px(f32),
    Percent(f32),
}

impl LayoutValue {
    /// Converts the layout value to pixels based on the given reference size.
    pub fn to_pixels(&self, reference: f32) -> f32 {
        match self {
            LayoutValue::Px(px) => *px,
            LayoutValue::Percent(percent) => reference * (*percent / 100.0),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LayoutReference {
    Parent,
    Child,
    #[default]
    Available,
}

impl LayoutReference {
    pub fn solve(&self, parent: f32, child: f32, available: f32) -> f32 {
        match self {
            LayoutReference::Parent => parent,
            LayoutReference::Child => child,
            LayoutReference::Available => available,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutAxis {
    Horizontal,
    Vertical,
}

impl LayoutAxis {
    /// Returns the orthogonal layout axis.
    pub fn orthogonal(&self) -> Self {
        match self {
            LayoutAxis::Horizontal => LayoutAxis::Vertical,
            LayoutAxis::Vertical => LayoutAxis::Horizontal,
        }
    }
}

impl Not for LayoutAxis {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.orthogonal()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutDirection {
    #[default]
    Start,
    End,
}

impl LayoutDirection {
    /// Returns the opposite layout direction.
    pub fn opposite(&self) -> Self {
        match self {
            LayoutDirection::Start => LayoutDirection::End,
            LayoutDirection::End => LayoutDirection::Start,
        }
    }
}

impl Not for LayoutDirection {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.opposite()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LayoutAlignment {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutWithCenter<V>
where
    V: Debug + Clone + PartialEq,
{
    Center,
    Value(V),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutHorizontalEdge {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutVerticalEdge {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
pub enum LayoutEdge {
    Horizontal(LayoutHorizontalEdge),
    Vertical(LayoutVerticalEdge),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutPivot(
    pub LayoutWithCenter<LayoutHorizontalEdge>,
    pub LayoutWithCenter<LayoutVerticalEdge>,
);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutPack<V>
where
    V: Debug + PartialEq,
{
    pub horizontal: V,
    pub vertical: V,
}

impl<V> LayoutPack<V>
where
    V: Debug + PartialEq,
{
    pub fn new(horizontal: V, vertical: V) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    pub fn axis(&self, axis: LayoutAxis) -> &V {
        match axis {
            LayoutAxis::Horizontal => &self.horizontal,
            LayoutAxis::Vertical => &self.vertical,
        }
    }
}
