//! UI navigation components.

use std::ops::Not;

/// The sign of a navigation action, either positive or negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)] // We won't ever have more signs
pub enum Sign {
    /// Towards positive direction.
    Positive,
    /// Towards negative direction.
    Negative,
}

/// Navigation axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        matches!(self, NavAxis::Horizontal)
    }

    /// Returns `true` if the axis is vertical.
    pub fn is_vertical(&self) -> bool {
        matches!(self, NavAxis::Vertical)
    }

    /// Flips the axis to its opposite.
    pub fn flip(&self) -> Self {
        match self {
            NavAxis::Horizontal => NavAxis::Vertical,
            NavAxis::Vertical => NavAxis::Horizontal,
        }
    }

    /// Returns the [`NavDirection`] for the given [`Sign`] on this axis.
    pub fn direction(&self, sign: Sign) -> NavDirection {
        match (self, sign) {
            (NavAxis::Horizontal, Sign::Positive) => NavDirection::Right,
            (NavAxis::Horizontal, Sign::Negative) => NavDirection::Left,
            (NavAxis::Vertical, Sign::Positive) => NavDirection::Down,
            (NavAxis::Vertical, Sign::Negative) => NavDirection::Up,
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
            NavDirection::Up | NavDirection::Down => NavAxis::Vertical,
            NavDirection::Left | NavDirection::Right => NavAxis::Horizontal,
        }
    }

    /// Flips the direction to its opposite.
    pub fn flip(&self) -> Self {
        match self {
            NavDirection::Up => NavDirection::Down,
            NavDirection::Down => NavDirection::Up,
            NavDirection::Left => NavDirection::Right,
            NavDirection::Right => NavDirection::Left,
        }
    }

    /// The [`Sign`] of this direction.
    pub fn sign(&self) -> Sign {
        match self {
            NavDirection::Up | NavDirection::Left => Sign::Negative,
            NavDirection::Down | NavDirection::Right => Sign::Positive,
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
