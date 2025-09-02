//! Direction related types.

use std::fmt;

use glam::IVec3;

macro_rules! directions {
    ($($i:literal => $dir:ident: $doc:literal {
        opposite: $oppo:ident,
        vec: ($x:literal, $y:literal, $z:literal),
        axis: $axis:ident,
        axis_dir: $axis_dir:ident,
        name: $name:literal,
        $(,)?
    }),*$(,)?) => {
        #[doc = "An enum representing 6 cardinal directions."]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[allow(clippy::exhaustive_enums)]
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

        #[cfg(feature = "serde")]
        impl serde::Serialize for Direction {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match self {
                    $(Self::$dir => serializer.serialize_str($name)),*
                }
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for Direction {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct Visitor;

                impl serde::de::Visitor<'_> for Visitor {
                    type Value = Direction;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "a direction name")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            $($name => Ok(Direction::$dir),)*
                            _ => Err(serde::de::Error::unknown_variant(value, &[$($name),*])),
                        }
                    }
                }

                deserializer.deserialize_str(Visitor)
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
        name: "down",
    },
    1 => Up: "The positive Y direction." {
        opposite: Down,
        vec: (0, 1, 0),
        axis: Y,
        axis_dir: Positive,
        name: "up",
    },
    2 => North: "The negative Z direction." {
        opposite: South,
        vec: (0, 0, -1),
        axis: Z,
        axis_dir: Negative,
        name: "north",
    },
    3 => South: "The positive Z direction." {
        opposite: North,
        vec: (0, 0, 1),
        axis: Z,
        axis_dir: Positive,
        name: "south",
    },
    4 => West: "The negative X direction." {
        opposite: East,
        vec: (-1, 0, 0),
        axis: X,
        axis_dir: Negative,
        name: "west",
    },
    5 => East: "The positive X direction." {
        opposite: West,
        vec: (1, 0, 0),
        axis: X,
        axis_dir: Positive,
        name: "east",
    },
}

impl Direction {
    /// The number of directions.
    pub const COUNT: usize = 6;

    /// Gets the ordinal of the direction.
    #[inline]
    pub fn ordinal(self) -> usize {
        self as u8 as usize
    }

    /// Gets the axis of the direction.
    #[inline]
    pub fn axis(self) -> Axis {
        self.into()
    }

    /// Gets the axis direction of the direction.
    #[inline]
    #[doc(alias = "axis_direction")]
    pub fn axis_dir(self) -> AxisDirection {
        self.into()
    }
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
    /// All available values of [`Axis`].
    pub const VALUES: [Self; 3] = [Self::X, Self::Y, Self::Z];

    /// Chooses a value from a position based on the axis.
    #[inline(always)]
    pub fn choose<T>(self, x: T, y: T, z: T) -> T {
        match self {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
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

/// An enum representing 4 cardinal directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::exhaustive_enums)]
#[repr(u8)]
pub enum EightWayDirection {
    /// Represents [`Direction::North`].
    North = 0,
    /// Represents [`Direction::South`] and [`Direction::East`].
    NorthEast,
    /// Represents [`Direction::East`].
    East,
    /// Represents [`Direction::South`] and [`Direction::West`].
    SouthEast,
    /// Represents [`Direction::South`].
    South,
    /// Represents [`Direction::South`] and [`Direction::West`].
    SouthWest,
    /// Represents [`Direction::West`].
    West,
    /// Represents [`Direction::North`] and [`Direction::West`].
    NorthWest,
}

impl EightWayDirection {
    /// The amount of directions.
    pub const COUNT: usize = 8;

    /// All directions.
    pub const ALL: [Self; Self::COUNT] = [
        Self::North,
        Self::NorthEast,
        Self::East,
        Self::SouthEast,
        Self::South,
        Self::SouthWest,
        Self::West,
        Self::NorthWest,
    ];

    /// Gets the [`Direction`]s of this direction.
    pub const fn directions(self) -> &'static [Direction] {
        match self {
            Self::North => &[Direction::North],
            Self::NorthEast => &[Direction::North, Direction::East],
            Self::East => &[Direction::East],
            Self::SouthEast => &[Direction::South, Direction::East],
            Self::South => &[Direction::South],
            Self::SouthWest => &[Direction::South, Direction::West],
            Self::West => &[Direction::West],
            Self::NorthWest => &[Direction::North, Direction::West],
        }
    }
}

impl From<Direction> for EightWayDirection {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::North => Self::North,
            Direction::East => Self::East,
            Direction::South => Self::South,
            Direction::West => Self::West,
            Direction::Up => Self::North,
            Direction::Down => Self::South,
        }
    }
}

impl From<EightWayDirection> for IVec3 {
    fn from(value: EightWayDirection) -> Self {
        value.directions().iter().copied().map(IVec3::from).sum()
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
            Self::InvalidId(id) => write!(f, "Invalid direction ID: {id}"),
        }
    }
}

impl std::error::Error for Error {}
