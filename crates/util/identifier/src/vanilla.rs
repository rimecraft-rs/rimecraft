//! Vanilla implementation of namespace and path.

use std::{hash::Hash, str::FromStr, sync::Arc};

use crate::Separate;

mod macros;

/// Namespace of an vanilla Minecraft `Identifier`.
///
/// This is the default value of a [`Namespace`].
pub const MINECRAFT: Namespace = Namespace(ArcCowStr::Ref("minecraft"));

/// Namespace of an vanilla `Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Namespace(ArcCowStr<'static>);

impl Namespace {
    /// Creates a new `Namespace` from the given value.
    ///
    /// # Panics
    ///
    /// Panics if the given namespace is invalid.
    #[inline]
    pub fn new<T>(value: T) -> Self
    where
        T: Into<Arc<str>>,
    {
        let value = value.into();
        validate_namespace(&value).unwrap();
        Self(ArcCowStr::Arc(value))
    }

    /// Creates a new `Namespace` from the given value
    /// at compile time.
    ///
    /// # Safety
    ///
    /// The given namespace shoule be all [a-z0-9_.-] character.
    pub const fn const_new(value: &'static str) -> Self {
        Self(ArcCowStr::Ref(value))
    }

    /// Returns the inner value of the `Namespace`.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for Namespace {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Namespace {
    type Err = Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_namespace(s)?;
        Ok(Self(ArcCowStr::Arc(s.into())))
    }
}

impl Separate for Namespace {
    const SEPARATOR: char = ':';
}

impl Default for Namespace {
    #[inline]
    fn default() -> Self {
        MINECRAFT.clone()
    }
}

/// Path of an vanilla `Identifier`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path(ArcCowStr<'static>);

impl Path {
    /// Creates a new [`Path`] from the given value.
    ///
    /// # Panics
    ///
    /// Panics if the given path is invalid.
    #[inline]
    pub fn new<T>(value: T) -> Self
    where
        T: Into<Arc<str>>,
    {
        Self::try_new(value).unwrap()
    }

    /// Creates a new [`Path`] from the given value.
    ///
    /// # Errors
    ///
    /// This function will return an error if the given path is invalid.
    pub fn try_new<T>(value: T) -> Result<Self, Error>
    where
        T: Into<Arc<str>>,
    {
        let value = value.into();
        validate_path(&value)?;
        Ok(Self(ArcCowStr::Arc(value)))
    }

    /// Creates a new [`Path`] from the given value.
    ///
    /// This function accepts a 2-dimension [`Vec`] which stands for words wrapped in locations.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rimecraft_identifier::vanilla::Path;
    /// let path = Path::new_formatted(vec![
    ///         vec!["tags"],
    ///         vec![],
    ///         vec!["piglin", "", "likes"],
    ///     ]);
    /// let identifier = Identifier::new(MINECRAFT, path);
    ///
    /// assert_eq!("minecraft:tags/piglin_likes", identifier.to_string());
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if any of the given words is invalid.
    #[inline]
    pub fn new_formatted<T>(values: Vec<Vec<T>>) -> Self
    where
        T: Into<Arc<str>>,
    {
        Self::try_new_formatted(values).unwrap()
    }

    /// Creates a new [`Path`] from the given value.
    ///
    /// This function accepts a 2-dimension [`Vec`] which stands for words wrapped in locations.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rimecraft_identifier::vanilla::Path;
    /// let path = Path::try_new_formatted(vec![
    ///         vec!["tags"],
    ///         vec![],
    ///         vec!["piglin", "", "repellents"],
    ///     ]).unwrap();
    /// let identifier = Identifier::new(MINECRAFT, path);
    /// assert_eq!("minecraft:tags/piglin_repellents", identifier.to_string());
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if the given path is invalid.
    #[inline]
    pub fn try_new_formatted<T>(values: Vec<Vec<T>>) -> Result<Self, Error>
    where
        T: Into<Arc<str>>,
    {
        let values: Vec<Vec<Result<Arc<str>, Error>>> = values
            .into_iter()
            .map(|v| {
                v.into_iter()
                    .map(|s| {
                        let s = s.into();
                        match validate_path(&s) {
                            Ok(_) => Ok(s),
                            Err(error) => Err(error),
                        }
                    })
                    .collect()
            })
            .collect();

        if let Some(err) = values
            .iter()
            .flat_map(|v| v.iter())
            .find_map(|r| r.as_ref().err())
            .cloned()
        {
            return Err(err);
        }

        let values: Vec<Vec<Arc<str>>> = values
            .into_iter()
            .filter_map(|v| {
                let v: Vec<Arc<str>> = v
                    .into_iter()
                    .filter_map(|r| r.ok().and_then(|s| (!s.is_empty()).then_some(s)))
                    .collect();

                (!v.is_empty()).then_some(v)
            })
            .collect();

        // Join words
        let values: Vec<Arc<str>> = values.into_iter().map(|v| v.join("_").into()).collect();

        // Join locations
        let arc: Arc<str> = values.join("/").into();

        Ok(Self(ArcCowStr::Arc(arc)))
    }

    /// Creates a new [`Path`] from the given value
    /// at compile time.
    ///
    /// The given path should be all [a-z0-9/_.-] character.
    #[inline]
    pub const fn new_unchecked(value: &'static str) -> Self {
        Self(ArcCowStr::Ref(value))
    }

    /// Returns the inner value of the [`Path`].
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for Path {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Path {
    type Err = Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_path(s)?;
        Ok(Self(ArcCowStr::Arc(s.into())))
    }
}

#[derive(Debug, Clone)]
enum ArcCowStr<'a> {
    Arc(Arc<str>),
    Ref(&'a str),
}

impl std::fmt::Display for ArcCowStr<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArcCowStr::Arc(s) => s.fmt(f),
            ArcCowStr::Ref(s) => s.fmt(f),
        }
    }
}

impl Hash for ArcCowStr<'_> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            ArcCowStr::Arc(s) => (**s).hash(state),
            ArcCowStr::Ref(s) => (**s).hash(state),
        }
    }
}

impl PartialEq for ArcCowStr<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ArcCowStr::Arc(s), ArcCowStr::Arc(o)) => s == o,
            (ArcCowStr::Ref(s), ArcCowStr::Ref(o)) => s == o,
            (ArcCowStr::Arc(s), ArcCowStr::Ref(o)) => **s == **o,
            (ArcCowStr::Ref(s), ArcCowStr::Arc(o)) => **s == **o,
        }
    }
}

impl Eq for ArcCowStr<'_> {}

impl PartialOrd for ArcCowStr<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ArcCowStr<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (ArcCowStr::Arc(s), ArcCowStr::Arc(o)) => s.cmp(o),
            (ArcCowStr::Ref(s), ArcCowStr::Ref(o)) => s.cmp(o),
            (ArcCowStr::Arc(s), ArcCowStr::Ref(o)) => (**s).cmp(*o),
            (ArcCowStr::Ref(s), ArcCowStr::Arc(o)) => (**s).cmp(&**o),
        }
    }
}

impl AsRef<str> for ArcCowStr<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        match self {
            ArcCowStr::Arc(s) => s.as_ref(),
            ArcCowStr::Ref(s) => s,
        }
    }
}

fn validate_namespace(value: &str) -> Result<(), Error> {
    for c in value.chars() {
        if !matches!(c, '_' | '-' | 'a'..='z' | '0'..='9' | '.') {
            return Err(Error::InvalidNamespace(value.to_owned()));
        }
    }

    Ok(())
}

fn validate_path(value: &str) -> Result<(), Error> {
    for c in value.chars() {
        if !matches!(c, '_' | '-' | '/' | 'a'..='z' | '0'..='9' | '.') {
            return Err(Error::InvalidPath(value.to_owned()));
        }
    }

    Ok(())
}

/// Error type for `Namespace` and `Path`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Error {
    /// The given namespace is invalid.
    InvalidNamespace(String),
    /// The given path is invalid.
    InvalidPath(String),
}

impl std::fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidNamespace(s) => write!(f, "invalid namespace: {}", s),
            Error::InvalidPath(s) => write!(f, "invalid path: {}", s),
        }
    }
}

/// Identifier of vanilla.
#[doc(alias = "ResourceLocation")]
pub type Identifier = crate::Identifier<Namespace, Path>;

#[cfg(test)]
mod tests {
    use crate::vanilla::{Namespace, Path};

    use super::Identifier;

    #[test]
    fn display() {
        let id = Identifier::new(Namespace::new("foo"), Path::new("bar"));
        assert_eq!(id.to_string(), "foo:bar");
    }

    #[test]
    fn parse() {
        let id: Identifier = "foo:bar".parse().unwrap();
        assert_eq!(id, Identifier::new(Namespace::new("foo"), Path::new("bar")));
    }
}

#[cfg(feature = "vanilla-registry")]
impl ::rimecraft_registry::key::Root for Identifier {
    #[inline]
    fn root() -> Self {
        Self::new(Default::default(), Path::new("root"))
    }
}
