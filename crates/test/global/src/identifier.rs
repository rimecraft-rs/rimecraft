//! Identifier wrappers.

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use serde::Serialize;

pub use identifier as raw;

/// An identifier.
#[derive(PartialEq, Eq, Clone, Hash)]
pub struct Id(#[doc(hidden)] pub identifier::vanilla::Identifier);

impl Id {
    /// Creates a new identifier at compile time.
    ///
    /// # Safety
    ///
    /// The namespace and path should be valid in vanilla minecraft.
    pub const unsafe fn const_new(namespace: &'static str, path: &'static str) -> Self { unsafe {
        Self(identifier::vanilla::Identifier::new(
            identifier::vanilla::Namespace::new_unchecked(namespace),
            identifier::vanilla::Path::new_unchecked(path),
        ))
    }}
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl FromStr for Id {
    type Err = <identifier::vanilla::Identifier as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        identifier::vanilla::Identifier::from_str(s).map(Self)
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        identifier::vanilla::Identifier::deserialize(deserializer).map(Self)
    }
}
