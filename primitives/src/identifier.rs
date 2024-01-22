/// Used for caching namespaces at runtime.
#[cfg(feature = "caches")]
static NAMESPACE_CACHES: once_cell::sync::Lazy<rimecraft_caches::Caches<String>> =
    once_cell::sync::Lazy::new(rimecraft_caches::Caches::new);

const DEFAULT_NAMESPACE: &str = "rimecraft";

/// An identifier used to identify things,
/// containing a namespace and a path.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.util.Identifier` (yarn).
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct Identifier {
    #[cfg(feature = "caches")]
    namespace: crate::Ref<'static, String>,

    #[cfg(not(feature = "caches"))]
    namespace: String,

    path: String,
}

impl Identifier {
    /// Creates a new identifier.
    ///
    /// # Panics
    ///
    /// Panics when either namespace or path contains
    /// non-[a-z0-9/._-] characters.
    #[inline]
    pub fn new(namespace: String, path: String) -> Self {
        Self::try_new(namespace, path).unwrap()
    }

    /// Creates a new identifier.
    #[cfg(feature = "caches")]
    pub fn try_new(namespace: String, path: String) -> Result<Self, Error> {
        if is_path_valid(&path) {
            if !NAMESPACE_CACHES.contains(&namespace) && !is_namespace_valid(&namespace) {
                return Err(Error::InvalidChars { namespace, path });
            }

            Ok(Self {
                namespace: crate::Ref(NAMESPACE_CACHES.get(namespace)),
                path,
            })
        } else {
            Err(Error::InvalidChars { namespace, path })
        }
    }

    /// Creates a new identifier.
    #[cfg(not(feature = "caches"))]
    pub fn try_new(namespace: &str, path: String) -> Result<Self, Error> {
        if Self::is_path_valid(&path) && Self::is_namespace_valid(namespace) {
            Ok(Self {
                namespace: namespace.to_owned(),
                path,
            })
        } else {
            Err(Error::InvalidChars {
                namespace: namespace.to_owned(),
                path,
            })
        }
    }

    /// Parse a string identifier (ex. `minecraft:air`).
    ///
    /// # Panics
    ///
    /// Panics when either namespace or path contains
    /// non-[a-z0-9/._-] characters.
    #[inline]
    pub fn parse(id: &str) -> Self {
        Self::try_parse(id).unwrap()
    }

    /// Parse a string identifier (ex. `minecraft:air`).
    #[inline]
    pub fn try_parse(id: &str) -> Result<Self, Error> {
        Self::split(id, ':')
    }

    /// Splits the `id` into an array of two strings at the first occurrence
    /// of `delimiter`, excluding the delimiter character, or uses `:` for
    /// the first string in the resulting array when the deliminator does
    /// not exist or is the first character.
    fn split(id: &str, delimiter: char) -> Result<Self, Error> {
        if let Some(arr) = id.split_once(delimiter) {
            Self::try_new(arr.0.to_owned(), arr.1.to_owned())
        } else {
            Self::try_new(DEFAULT_NAMESPACE.to_owned(), id.to_owned())
        }
    }

    /// Gets the namespace of this id.
    #[inline]
    pub fn namespace(&self) -> &str {
        #[cfg(feature = "caches")]
        return self.namespace.0;

        #[cfg(not(feature = "caches"))]
        return &self.namespace;
    }

    /// Gets the path of this id.
    #[inline]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Trimming the formatting of this identifier.
    ///
    /// # Examples
    /// ```
    /// # use rimecraft_primitives::id;
    /// assert_eq!(id!("rimecraft", "gold_ingot").trim_fmt(), "gold_ingot");
    /// ```
    #[inline]
    pub fn trim_fmt(&self) -> String {
        if *self.namespace == DEFAULT_NAMESPACE {
            self.path.to_owned()
        } else {
            self.to_string()
        }
    }
}

/// Creates a new [`Identifier`].
///
/// # Examples
///
/// ```
/// # use rimecraft_primitives::id;
/// // Either parse or create an identifier directly.
/// assert_eq!(id!("namespace:path").to_string(), "namespace:path");
/// assert_eq!(id!("namespace", "path").to_string(), "namespace:path");
///
/// // By default, the namespace is rimecraft.
/// assert_eq!(id!("path").to_string(), "rimecraft:path");
///
/// // Concat paths with slashes.
/// assert_eq!(id!["namespace", "path", "path1", "path2"].to_string(), "namespace:path/path1/path2");
/// ```
#[macro_export]
macro_rules! id {
    ($ns:expr, $p:expr) => {
        {
            $crate::identifier::Identifier::new($ns.to_string(), $p.to_string())
        }
    };

    ($ns:expr, $p:expr, $( $pp:expr ),+) => {
        {
            let mut path = String::new();
            path.push_str($p.as_ref());
            $(
                path.push('/');
                path.push_str($pp.as_ref());
            )*
            $crate::identifier::Identifier::new($ns.to_string(), path)
        }
    };

    ($n:expr) => {
        {
            $crate::identifier::Identifier::parse($n.as_ref())
        }
    }
}

/// Whether `namespace` can be used as an identifier's namespace
pub fn is_namespace_valid(namespace: &str) -> bool {
    for c in namespace.chars() {
        if !(c == '_' || c == '-' || c >= 'a' || c <= 'z' || c >= '0' || c <= '9' || c == '.') {
            return false;
        }
    }
    true
}

/// Whether `path` can be used as an identifier's path
pub fn is_path_valid(path: &str) -> bool {
    for c in path.chars() {
        if !(c == '_'
            || c == '-'
            || c >= 'a'
            || c <= 'z'
            || c >= '0'
            || c <= '9'
            || c == '.'
            || c == '/')
        {
            return false;
        }
    }
    true
}

/// Error variants of [`Identifier`].
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Invalid characters in either namespace or path.
    #[error("non [a-z0-9/._-] character in id {namespace}:{path}")]
    InvalidChars { namespace: String, path: String },
}

impl std::fmt::Display for Identifier {
    /// Stringify this identifier as `namespace:path` format.
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "caches")]
        return write!(f, "{}:{}", self.namespace.0, self.path);

        #[cfg(not(feature = "caches"))]
        return write!(f, "{}:{}", self.namespace, self.path);
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Identifier {
    /// Serialize this identifier as string.
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Identifier {
    /// Deserialize this identifier from string.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let str = String::deserialize(deserializer)?;

        Self::try_parse(str.as_str()).map_err(|_| {
            D::Error::invalid_value(
                serde::de::Unexpected::Str(str.as_str()),
                &"string with a ':' separated and which chars are in [a-z0-9/._-]",
            )
        })
    }
}

#[cfg(feature = "edcode")]
impl rimecraft_edcode::Encode for Identifier {
    type Error = std::convert::Infallible;

    #[inline]
    fn encode<B>(&self, buf: &mut B) -> Result<(), Self::Error>
    where
        B: bytes::BufMut,
    {
        self.to_string().encode(buf)
    }
}

#[cfg(feature = "edcode")]
impl<'de> rimecraft_edcode::Decode<'de> for Identifier {
    type Output = Self;

    type Error = rimecraft_edcode::error::EitherError<
        Error,
        rimecraft_edcode::error::ErrorWithVarI32Err<std::string::FromUtf8Error>,
    >;

    #[inline]
    fn decode<B>(buf: &'de mut B) -> Result<Self::Output, Self::Error>
    where
        B: bytes::Buf,
    {
        use rimecraft_edcode::error::EitherError;
        String::decode(buf)
            .map_err(EitherError::B)
            .and_then(|s| Self::try_parse(&s).map_err(EitherError::A))
    }
}
