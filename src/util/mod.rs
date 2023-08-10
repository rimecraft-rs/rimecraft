use std::{hash::Hash, ops::Deref};

pub mod collections;
pub mod math;

static IDENTIFIER_NAMESPACE_CACHES: crate::collections::Caches<String> =
    crate::collections::Caches::new();

/// An identifier used to identify things.
///
/// This is also known as "resource location", "namespaced ID",
/// "location", or "Identifier".
/// This is a non-typed immutable object, and identifies things
/// using a combination of namespace and path.
#[derive(PartialEq, Eq, Clone, Hash)]
pub struct Id {
    namespace: Ref<'static, String>,
    path: String,
}

impl Id {
    pub fn new(namespace: &str, path: &str) -> anyhow::Result<Self> {
        if Self::is_namespace_valid(namespace) && Self::is_path_valid(path) {
            Ok(Self {
                namespace: Ref(IDENTIFIER_NAMESPACE_CACHES.get(namespace.to_string())),
                path: path.to_string(),
            })
        } else {
            Err(anyhow::anyhow!(
                "Non [a-z0-9/._-] character in id {namespace}:{path}"
            ))
        }
    }

    pub fn parse(id: &str) -> Self {
        Self::try_parse(id).unwrap()
    }

    pub fn try_parse(id: &str) -> anyhow::Result<Self> {
        Self::split_on(id, ':')
    }

    pub fn split_on(id: &str, delimiter: char) -> anyhow::Result<Self> {
        match id.split_once(delimiter) {
            Some(arr) => Self::new(arr.0, arr.1),
            None => Self::new("rimecraft", id),
        }
    }

    fn is_namespace_valid(namespace: &str) -> bool {
        for c in namespace.chars() {
            if !(c == '_' || c == '-' || c >= 'a' || c <= 'z' || c >= '0' || c <= '9' || c == '.') {
                return false;
            }
        }
        true
    }

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

    pub fn namespace(&self) -> &'static str {
        self.namespace.0
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.namespace)?;
        f.write_str(":")?;
        f.write_str(&self.path)
    }
}

impl serde::Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Id {
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

#[cfg(test)]
mod id_tests {
    use crate::Id;

    #[test]
    fn to_str() {
        let id = Id::new("modid", "example_path").unwrap();
        assert_eq!(id.to_string(), "modid:example_path");
    }

    #[test]
    fn parse_str() {
        let raw = "modid:example_path";
        let id = Id::parse(raw);
        assert_eq!(id.to_string(), raw);
    }
}

/// Describes a var int.
pub struct VarInt(pub i32);

impl VarInt {
    pub fn len(self) -> usize {
        for i in 1..5 {
            if (self.0 & -1 << i * 7) == 0 {
                return i as usize;
            }
        }

        5
    }
}

/// Represents types of enum that can be itered with values, like Java.
pub trait EnumValues<const N: usize>: Sized + Clone + Copy + PartialEq + Eq {
    fn values() -> [Self; N];
}

/// Represents a reference with enhancements based on `&'a`.
pub struct Ref<'a, T: 'a + ?Sized>(pub &'a T);

impl<'a, T: 'a + ?Sized> Copy for Ref<'a, T> {}

impl<'a, T: 'a + ?Sized> Clone for Ref<'a, T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<'a, T: 'a + ?Sized> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T> From<T> for Ref<'static, T> {
    fn from(value: T) -> Self {
        Self(Box::leak(Box::new(value)))
    }
}

impl<'a, T: 'a> From<&'a T> for Ref<'a, T> {
    fn from(value: &'a T) -> Self {
        Self(value)
    }
}

impl<'a, T: 'a> Eq for Ref<'a, T> {}

impl<'a, T: 'a> PartialEq for Ref<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 as *const T as usize == other.0 as *const T as usize
    }
}

impl<'a, T: 'a> Hash for Ref<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as *const T as usize).hash(state)
    }
}

#[cfg(test)]
mod ref_tests {
    use super::Ref;

    #[test]
    fn aligned() {
        let string = "Hello, world!";
        let str_ref = Ref(string);

        assert_eq!(
            unsafe { &*(&str_ref as *const Ref<str> as *const &'static str) },
            &string
        );
    }
}

/// A static instance that can be created with a type in a [`std::sync::Mutex`]
/// to be mutable and be freezed into (maybe) another type inside a once cell.
/// Which the freezed instance can be accessed without a lock and be borrowed
/// outlives static.
///
/// The freezed instance can be accessed directly with the deref trait
/// implemented by this type.
pub struct Freezer<I, M = I>
where
    M: Freeze<I>,
{
    immutable: once_cell::sync::OnceCell<I>,
    /// The mutable instance.
    pub mutable: std::sync::Mutex<Option<M>>,
}

impl<I, M: Freeze<I>> Freezer<I, M> {
    pub const fn new(mutable: M) -> Self {
        Self {
            immutable: once_cell::sync::OnceCell::new(),
            mutable: std::sync::Mutex::new(Some(mutable)),
        }
    }

    /// Freeze this instance with provided options.
    pub fn freeze(&self, opts: M::Opts) {
        assert!(!self.is_freezed());
        let _ = self
            .immutable
            .set(self.mutable.lock().unwrap().take().unwrap().build(opts));
    }

    /// Whether this instance has been already freezed.
    pub fn is_freezed(&self) -> bool {
        self.immutable.get().is_some()
    }
}

impl<I, M: Freeze<I>> Deref for Freezer<I, M> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        unsafe { self.immutable.get_unchecked() }
    }
}

/// Describes a type that can be used for mutable instance (`M`) in a [`Freezer`].
/// The generic type `T` is the freeze output type of this type.
///
/// By default, all types will can be freezed into themselves
/// with empty tuple options.
pub trait Freeze<T> {
    /// Options for the freeze operation.
    type Opts;

    /// Build and freeze this value into `T` with options.
    fn build(self, opts: Self::Opts) -> T;
}

impl<T> Freeze<T> for T {
    type Opts = ();

    fn build(self, _opts: Self::Opts) -> T {
        self
    }
}
