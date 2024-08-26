use std::{fmt::Display, str::FromStr};

/// An identifier.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Id(pub(crate) identifier::vanilla::Identifier);

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Id {
    type Err = <identifier::vanilla::Identifier as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        identifier::vanilla::Identifier::from_str(s).map(Self)
    }
}
