pub mod fmt;
pub mod math;

use fmt::Formatting;
use serde::{Deserialize, Serialize};

use std::{fmt::UpperHex, ops::Deref, str::FromStr};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

impl From<Rarity> for Formatting {
    #[inline]
    fn from(value: Rarity) -> Self {
        match value {
            Rarity::Common => Formatting::White,
            Rarity::Uncommon => Formatting::Yellow,
            Rarity::Rare => Formatting::Aqua,
            Rarity::Epic => Formatting::LightPurple,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Hand {
    Main,
    Off,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct RGB {
    value: u32,
}

impl RGB {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    #[inline]
    pub fn value(self) -> u32 {
        self.value
    }
}

impl UpperHex for RGB {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:06X}", self.value)
    }
}

impl FromStr for RGB {
    type Err = std::num::ParseIntError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(RGB::new)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Clone, Copy)]
pub struct Stringified<T>(pub T);

impl<T> Deref for Stringified<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Serialize for Stringified<T>
where
    T: ToString,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de, T> Deserialize<'de> for Stringified<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        String::deserialize(deserializer)?
            .parse()
            .map(Stringified)
            .map_err(D::Error::custom)
    }
}
