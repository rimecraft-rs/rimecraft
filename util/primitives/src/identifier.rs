/// Used for caching namespaces in runtime.
#[cfg(feature = "caches")]
static NAMESPACE_CACHES: once_cell::sync::Lazy<rimecraft_caches::Caches<String>> =
    once_cell::sync::Lazy::new(rimecraft_caches::Caches::new);

/// An identifier used to identify things,
/// containing a namespace and a path.
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct Identifier {
    #[cfg(feature = "caches")]
    namespace: super::Ref<'static, String>,

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
    pub fn new(namespace: &str, path: String) -> Self {
        Self::try_new(namespace, path).unwrap()
    }

    /// Creates a new identifier.
    #[cfg(feature = "caches")]
    pub fn try_new(namespace: &str, path: String) -> Result<Self, Error> {
        let namespace_owned = namespace.to_owned();
        if Self::is_path_valid(&path) {
            if !NAMESPACE_CACHES.contains(&namespace_owned) && !Self::is_namespace_valid(namespace)
            {
                return Err(Error::InvalidChars {
                    namespace: namespace_owned,
                    path,
                });
            }

            Ok(Self {
                namespace: super::Ref(NAMESPACE_CACHES.get(namespace_owned)),
                path,
            })
        } else {
            Err(Error::InvalidChars {
                namespace: namespace_owned,
                path,
            })
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
        Self::split_on(id, ':')
    }

    /// Split a string identifier based on a custom
    /// delimiter.
    fn split_on(id: &str, delimiter: char) -> Result<Self, Error> {
        if let Some(arr) = id.split_once(delimiter) {
            Self::try_new(arr.0, arr.1.to_owned())
        } else {
            Self::try_new("unknown", id.to_owned())
        }
    }

    #[inline]
    fn is_namespace_valid(namespace: &str) -> bool {
        for c in namespace.chars() {
            if !(c == '_' || c == '-' || c >= 'a' || c <= 'z' || c >= '0' || c <= '9' || c == '.') {
                return false;
            }
        }
        true
    }

    #[inline]
    fn is_path_valid(path: &str) -> bool {
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
        return write!(f, "{}:{}", &*self.namespace, self.path);

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
