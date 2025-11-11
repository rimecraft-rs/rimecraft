//! UI navigation components.

pub mod gui;
pub mod screen;

use std::ops::Not;

/// The sign of a navigation action, either positive or negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::exhaustive_enums)] // We won't ever have more signs
pub enum Sign {
    /// Towards positive direction.
    Positive,
    /// Towards negative direction.
    Negative,
}

/// Navigation axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::exhaustive_enums)] // We won't ever have more axes
pub enum NavAxis {
    /// The horizontal axis.
    Horizontal,
    /// The vertical axis.
    Vertical,
}

impl NavAxis {
    /// Returns `true` if the axis is horizontal.
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal)
    }

    /// Returns `true` if the axis is vertical.
    pub fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
    }

    /// Flips the axis to its opposite.
    pub fn flip(&self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }

    /// Returns the [`NavDirection`] for the given [`Sign`] on this axis.
    pub fn direction(&self, sign: Sign) -> NavDirection {
        match (self, sign) {
            (Self::Horizontal, Sign::Positive) => NavDirection::Right,
            (Self::Horizontal, Sign::Negative) => NavDirection::Left,
            (Self::Vertical, Sign::Positive) => NavDirection::Down,
            (Self::Vertical, Sign::Negative) => NavDirection::Up,
        }
    }
}

impl Not for NavAxis {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.flip()
    }
}

/// Navigation directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::exhaustive_enums)] // We won't ever have more directions
pub enum NavDirection {
    /// Navigates up.
    Up,
    /// Navigates down.
    Down,
    /// Navigates left.
    Left,
    /// Navigates right.
    Right,
}

impl NavDirection {
    /// The [`NavAxis`] of this direction.
    pub fn axis(&self) -> NavAxis {
        match self {
            Self::Up | Self::Down => NavAxis::Vertical,
            Self::Left | Self::Right => NavAxis::Horizontal,
        }
    }

    /// Flips the direction to its opposite.
    pub fn flip(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// The [`Sign`] of this direction.
    pub fn sign(&self) -> Sign {
        match self {
            Self::Up | Self::Left => Sign::Negative,
            Self::Down | Self::Right => Sign::Positive,
        }
    }
}

impl NavDirection {
    /// Sorts the given coordinates in place according to this direction.
    ///
    /// If the direction is positive, sorts in ascending order. Otherwise, sorts in descending order.
    ///
    /// # Panics
    ///
    /// Panics if the coordinates cannot be compared.
    pub fn sort<V>(&self, coords: &mut [V])
    where
        V: PartialOrd,
    {
        if matches!(self.sign(), Sign::Positive) {
            coords.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            coords.sort_by(|a, b| b.partial_cmp(a).unwrap());
        }
    }
}

impl Not for NavDirection {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.flip()
    }
}

/// A component that has a navigation index.
pub trait WithNavIndex {
    /// Returns the navigation index of this component.
    fn nav_index(&self) -> Option<usize>;
}
