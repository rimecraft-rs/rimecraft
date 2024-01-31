//! Rust implementation of Minecraft resource location.

use std::{fmt::Display, str::FromStr};

#[cfg(feature = "vanilla")]
pub mod vanilla;

/// An identifier used to identify things.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier<N, P> {
    namespace: N,
    path: P,
}

impl<N, P> Identifier<N, P> {
    /// Creates a new identifier.
    #[inline]
    pub const fn new(namespace: N, path: P) -> Self {
        Self { namespace, path }
    }

    /// Gets the namespace of the identifier.
    #[inline]
    pub fn namespace(&self) -> &N {
        &self.namespace
    }

    /// Gets the path of the identifier.
    #[inline]
    pub fn path(&self) -> &P {
        &self.path
    }
}

/// Namespace types that is able to separate
/// with paths using a separator.
pub trait Separate {
    //TODO: use `Pattern` when it's stable

    /// The separator used to separate namespace and path.
    const SEPARATOR: char;
}

impl<N, P> Display for Identifier<N, P>
where
    N: Display + Separate,
    P: Display,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.namespace, N::SEPARATOR, self.path)
    }
}

/// Errors that may occur when parsing an identifier.
#[derive(Debug)]
pub enum FromStrError<EN, EP> {
    /// An error occurred when parsing the namespace.
    Namespace(EN),
    /// An error occurred when parsing the path.
    Path(EP),
    /// The separator is not found.
    Separate,
}

impl<EN, EP> Display for FromStrError<EN, EP>
where
    EN: Display,
    EP: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FromStrError::Namespace(err) => write!(f, "parse namespace: {}", err),
            FromStrError::Path(err) => write!(f, "parse path: {}", err),
            FromStrError::Separate => write!(f, "separator not found"),
        }
    }
}

impl<EN, EP> std::error::Error for FromStrError<EN, EP>
where
    EN: std::error::Error,
    EP: std::error::Error,
{
}

impl<N, P> FromStr for Identifier<N, P>
where
    N: FromStr + Separate,
    P: FromStr,
{
    type Err = FromStrError<N::Err, P::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (n, p) = s
            .split_once(<N as Separate>::SEPARATOR)
            .ok_or(FromStrError::Separate)?;
        Ok(Self::new(
            n.parse().map_err(FromStrError::Namespace)?,
            p.parse().map_err(FromStrError::Path)?,
        ))
    }
}

#[cfg(feature = "serde")]
mod serde {
    use std::{fmt::Display, str::FromStr};

    use serde::Serialize;

    use crate::{Identifier, Separate};

    impl<N, P> Serialize for Identifier<N, P>
    where
        N: Display + Separate,
        P: Display,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.to_string().serialize(serializer)
        }
    }

    impl<'de, N, P> serde::Deserialize<'de> for Identifier<N, P>
    where
        N: FromStr + Separate,
        P: FromStr,
    {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let str = <&'de str>::deserialize(deserializer)?;

            Self::from_str(str).map_err(|_| {
                serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(str),
                    &"identifier with a separator separated",
                )
            })
        }
    }
}

#[cfg(feature = "edcode")]
mod edcode {
    use std::{fmt::Display, str::FromStr};

    use rimecraft_edcode::{error::EitherError, Encode};

    use crate::{Identifier, Separate};

    impl<N, P> Encode for Identifier<N, P>
    where
        N: Display + Separate,
        P: Display,
    {
        type Error = std::convert::Infallible;

        #[inline]
        fn encode<B>(&self, buf: B) -> Result<(), Self::Error>
        where
            B: rimecraft_edcode::bytes::BufMut,
        {
            self.to_string().encode(buf)
        }
    }

    impl<N, P> rimecraft_edcode::Decode for Identifier<N, P>
    where
        N: FromStr + Separate,
        P: FromStr,
    {
        type Output = Self;

        type Error = EitherError<
            rimecraft_edcode::error::ErrorWithVarI32Err<std::string::FromUtf8Error>,
            crate::FromStrError<N::Err, P::Err>,
        >;

        #[inline]
        fn decode<B>(buf: B) -> Result<Self::Output, Self::Error>
        where
            B: rimecraft_edcode::bytes::Buf,
        {
            let str = <String>::decode(buf).map_err(EitherError::A)?;
            Self::from_str(str.as_str()).map_err(rimecraft_edcode::error::EitherError::B)
        }
    }
}
