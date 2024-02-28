//! Direction related types.

use std::fmt;

use glam::IVec3;

macro_rules! directions {
    ($($i:literal => $dir:ident: $doc:literal {
        opposite: $oppo:ident,
        vec: ($x:literal, $y:literal, $z:literal),
        axis: $axis:ident,
        axis_dir: $axis_dir:ident,
        $(,)?
    }),*$(,)?) => {
        #[doc = "An enum representing 6 cardinal directions."]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[allow(clippy::exhaustive_enums)]
        #[cfg_attr(
            feature = "serde",
            derive(serde::Serialize, serde::Deserialize),
            serde(rename_all = "lowercase")
        )]
        #[repr(u8)]
        pub enum Direction {
            $(
                #[doc = $doc]
                $dir = $i,
            )*
        }

        impl Direction {
            /// All directions.
            pub const ALL: [Self; 6] = [
                $(Self::$dir,)*
            ];

            /// Gets the opposite direction.
            #[inline]
            pub const fn opposite(self) -> Self {
                match self {
                    $(Self::$dir => Self::$oppo),*
                }
            }
        }

        impl TryFrom<u8> for Direction {
            type Error = Error;

            #[inline]
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $($i => Ok(Self::$dir),)*
                    _ => Err(Error::InvalidId(value)),
                }
            }
        }

        impl From<Direction> for u8 {
            #[inline]
            fn from(value: Direction) -> Self {
                value as u8
            }
        }

        impl From<Direction> for IVec3 {
            #[inline]
            fn from(value: Direction) -> Self {
                match value {
                    $(Direction::$dir => IVec3::new($x, $y, $z),)*
                }
            }
        }

        impl From<Direction> for Axis {
            #[inline]
            fn from(value: Direction) -> Self {
                match value {
                    $(Direction::$dir => Axis::$axis,)*
                }
            }
        }

        impl From<Direction> for AxisDirection {
            #[inline]
            fn from(value: Direction) -> Self {
                match value {
                    $(Direction::$dir => AxisDirection::$axis_dir,)*
                }
            }
        }

        impl From<(Axis, AxisDirection)> for Direction {
            #[inline]
            fn from((axis, dir): (Axis, AxisDirection)) -> Self {
                match (axis, dir) {
                    $( (Axis::$axis, AxisDirection::$axis_dir) => Direction::$dir, )*
                }
            }
        }
    };
}

directions! {
    0 => Down: "The negative Y direction." {
        opposite: Up,
        vec: (0, -1, 0),
        axis: Y,
        axis_dir: Negative,
    },
    1 => Up: "The positive Y direction." {
        opposite: Down,
        vec: (0, 1, 0),
        axis: Y,
        axis_dir: Positive,
    },
    2 => North: "The negative Z direction." {
        opposite: South,
        vec: (0, 0, -1),
        axis: Z,
        axis_dir: Negative,
    },
    3 => South: "The positive Z direction." {
        opposite: North,
        vec: (0, 0, 1),
        axis: Z,
        axis_dir: Positive,
    },
    4 => West: "The negative X direction." {
        opposite: East,
        vec: (-1, 0, 0),
        axis: X,
        axis_dir: Negative,
    },
    5 => East: "The positive X direction." {
        opposite: West,
        vec: (1, 0, 0),
        axis: X,
        axis_dir: Positive,
    },
}

impl From<(AxisDirection, Axis)> for Direction {
    #[inline]
    fn from((ad, a): (AxisDirection, Axis)) -> Self {
        (a, ad).into()
    }
}

/// An enum representing 3 cardinal axes.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[allow(clippy::exhaustive_enums)]
#[repr(u8)]
pub enum Axis {
    /// The X axis.
    X,
    /// The Y axis.
    Y,
    /// The Z axis.
    Z,
}

impl Axis {
    /// Chooses a value from a position based on the axis.
    #[inline]
    pub fn choose<T>(self, pos: (T, T, T)) -> T {
        match self {
            Axis::X => pos.0,
            Axis::Y => pos.1,
            Axis::Z => pos.2,
        }
    }
}

/// An enum representing 2 axis directions.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[allow(clippy::exhaustive_enums)]
#[repr(u8)]
pub enum AxisDirection {
    /// The positive direction.
    Positive,
    /// The negative direction.
    Negative,
}

impl AxisDirection {
    /// Gets the offset of the direction.
    #[inline]
    pub const fn offset(self) -> i32 {
        match self {
            AxisDirection::Positive => 1,
            AxisDirection::Negative => -1,
        }
    }

    /// Gets the opposite direction.
    #[inline]
    pub const fn opposite(self) -> Self {
        match self {
            AxisDirection::Positive => AxisDirection::Negative,
            AxisDirection::Negative => AxisDirection::Positive,
        }
    }
}

/// An error that can occur when converting a direction.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The given ID is invalid.
    InvalidId(u8),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId(id) => write!(f, "Invalid direction ID: {}", id),
        }
    }
}

impl std::error::Error for Error {}
