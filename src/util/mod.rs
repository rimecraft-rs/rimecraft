use std::{hash::Hash, ops::Deref};

pub mod collections;
pub mod math;

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    pub fn new(namespace: &str, path: &str) -> anyhow::Result<Self> {
        if Self::is_namespace_valid(namespace) && Self::is_path_valid(path) {
            Ok(Self {
                namespace: namespace.to_string(),
                path: path.to_string(),
            })
        } else {
            Err(anyhow::anyhow!(
                "Non [a-z0-9/._-] character in identifier: {namespace}:{path}"
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

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.namespace)?;
        f.write_str(":")?;
        f.write_str(&self.path)?;
        std::fmt::Result::Ok(())
    }
}

impl serde::Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Identifier {
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

/// Describes a var int.
pub struct VarI32(pub i32);

/// Represents types of enum that can be itered with values, like Java.
pub trait EnumValues<const N: usize>: Sized + Clone + Copy + PartialEq + Eq {
    fn values() -> [Self; N];
}

pub struct StaticRef<T: 'static + ?Sized>(pub &'static T);

impl<T: 'static + ?Sized> Copy for StaticRef<T> {}

impl<T: 'static + ?Sized> Clone for StaticRef<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T: 'static + ?Sized> Deref for StaticRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T: 'static> From<T> for StaticRef<T> {
    fn from(value: T) -> Self {
        Self(Box::leak(Box::new(value)))
    }
}

impl<T: 'static> Eq for StaticRef<T> {}

impl<T: 'static> PartialEq for StaticRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 as *const T as usize == other.0 as *const T as usize
    }
}

pub struct FreezeLazy<I, M = I>
where
    M: Freeze<I>,
{
    immutable: once_cell::sync::OnceCell<I>,
    pub mutable: parking_lot::Mutex<Option<M>>,
}

impl<I, M: Freeze<I>> FreezeLazy<I, M> {
    pub const fn new(mutable: M) -> Self {
        Self {
            immutable: once_cell::sync::OnceCell::new(),
            mutable: parking_lot::Mutex::new(Some(mutable)),
        }
    }

    pub fn freeze(&self, opts: M::Opts) {
        assert!(!self.is_freezed());
        let _ = self
            .immutable
            .set(self.mutable.lock().take().unwrap().build(opts));
    }

    pub fn is_freezed(&self) -> bool {
        self.immutable.get().is_some()
    }
}

impl<I, M: Freeze<I>> Deref for FreezeLazy<I, M> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        unsafe { self.immutable.get_unchecked() }
    }
}

pub trait Freeze<T> {
    type Opts;

    fn build(self, opts: Self::Opts) -> T;
}

impl<T> Freeze<T> for T {
    type Opts = ();

    fn build(self, _opts: Self::Opts) -> T {
        self
    }
}
