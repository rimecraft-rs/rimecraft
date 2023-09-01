use std::{hash::Hash, ops::Deref};

pub mod collections;
mod magic_num;
pub mod math;

static ID_NAMESPACE_CACHES: crate::collections::Caches<String> = crate::collections::Caches::new();

/// An identifier used to identify things.
///
/// This is also known as "resource location", "namespaced ID",
/// "location", or "Identifier".
/// This is a non-typed immutable object, and identifies things
/// using a combination of namespace and path.
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct Id {
    namespace: Ref<'static, String>,
    path: String,
}

impl Id {
    pub fn new(namespace: &str, path: String) -> Self {
        Self::try_new(namespace, path).unwrap()
    }

    pub fn try_new(namespace: &str, path: String) -> Result<Self, IdError> {
        let owned_namespace = namespace.to_string();
        if Self::is_path_valid(&path) {
            if !ID_NAMESPACE_CACHES.contains(&owned_namespace)
                && !Self::is_namespace_valid(namespace)
            {
                return Err(IdError::InvalidChars {
                    namespace: owned_namespace,
                    path,
                });
            }

            Ok(Self {
                namespace: Ref(ID_NAMESPACE_CACHES.get(owned_namespace)),
                path,
            })
        } else {
            Err(IdError::InvalidChars {
                namespace: owned_namespace,
                path,
            })
        }
    }

    pub fn parse(id: &str) -> Self {
        Self::try_parse(id).unwrap()
    }

    pub fn try_parse(id: &str) -> Result<Self, IdError> {
        Self::split_on(id, ':')
    }

    pub fn split_on(id: &str, delimiter: char) -> Result<Self, IdError> {
        match id.split_once(delimiter) {
            Some(arr) => Self::try_new(arr.0, arr.1.to_string()),
            None => Self::try_new("unknown", id.to_string()),
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

#[derive(thiserror::Error, Debug)]
pub enum IdError {
    #[error("non [a-z0-9/._-] character in id {namespace}:{path}")]
    InvalidChars { namespace: String, path: String },
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
        let id = Id::new("modid", "example_path".to_string());
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

#[cfg(test)]
mod var_int_tests {
    use super::VarInt;
    use crate::net::{Decode, Encode};

    #[test]
    fn encode_decode() {
        let num = 114514;

        let mut bytes_mut = bytes::BytesMut::new();
        VarInt(num).encode(&mut bytes_mut).unwrap();

        assert_eq!(bytes_mut.len(), VarInt(num).len());

        let mut bytes: bytes::Bytes = bytes_mut.into();
        assert_eq!(VarInt::decode(&mut bytes).unwrap(), num);
    }
}

/// Represents types of enum that can be itered with values, like Java.
pub trait EnumValues<const N: usize>: Sized + Clone + Copy + PartialEq + Eq {
    fn values() -> [Self; N];
}

/// Represents a reference with enhancements based on `&'a`.
#[derive(Debug)]
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

/// A type containing listeners of this event,
/// which can be invoked by an invoker.
///
/// The listeners are sorted by phases ([`i8`] by default)
/// that can be called in order.
///
/// This type was inspired by the event system in Fabric API.
pub struct Event<T, Phase = i8>
where
    T: ?Sized + 'static,
    Phase: Ord,
{
    /// Whether listeners has been modified before requesting the invoker.
    dirty: std::sync::atomic::AtomicBool,

    invoker_factory: fn(&'static [&'static T]) -> Box<T>,

    /// 0: raw listeners with phases
    /// 1: cached invoker
    /// 2: cached listener references
    listeners_and_cache:
        parking_lot::RwLock<(Vec<(Phase, *const T)>, Option<Box<T>>, Vec<&'static T>)>,
}

impl<T, Phase> Event<T, Phase>
where
    T: ?Sized,
    Phase: Ord,
{
    /// Create a new event with provided event factory.
    ///
    /// To avoid lifetime problems in the factory, listeners
    /// provied are all in static references so that they're
    /// able to be copied and moved.
    /// So you should add a `move` keyword before the closure
    /// to return in the factory.
    pub const fn new(factory: fn(&'static [&'static T]) -> Box<T>) -> Self {
        Self {
            listeners_and_cache: parking_lot::RwLock::new((Vec::new(), None, Vec::new())),
            invoker_factory: factory,
            dirty: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Get the invoker of this event.
    ///
    /// Once the invoker is created, it will be cached until
    /// the next modification of listeners, and will be re-created
    /// by the factory.
    pub fn invoker(&self) -> &T {
        if self.dirty.load(std::sync::atomic::Ordering::Acquire) {
            let mut write_guard = self.listeners_and_cache.write();
            write_guard.0.sort_by(|e0, e1| Phase::cmp(&e0.0, &e1.0));
            self.dirty
                .store(false, std::sync::atomic::Ordering::Release);

            write_guard.2 = write_guard.0.iter().map(|e| unsafe { &*e.1 }).collect();
            write_guard.1 = Some((self.invoker_factory)(unsafe {
                &*(&write_guard.2 as *const Vec<&'static T>)
            }));
        } else if self.listeners_and_cache.read().1.is_none() {
            let mut write_guard = self.listeners_and_cache.write();
            write_guard.1 = Some((self.invoker_factory)(unsafe {
                &*(&write_guard.2 as *const Vec<&'static T>)
            }));
        }

        unsafe { &*(&**self.listeners_and_cache.read().1.as_ref().unwrap() as *const T) }
    }

    /// Register a listener to this event for the specified phase.
    pub fn register_with_phase(&mut self, listener: Box<T>, phase: Phase) {
        self.listeners_and_cache
            .get_mut()
            .0
            .push((phase, Box::leak(listener)));

        if !self.dirty.load(std::sync::atomic::Ordering::Acquire) {
            self.dirty.store(true, std::sync::atomic::Ordering::Release);
        }
    }
}

impl<T, Phase> Event<T, Phase>
where
    T: ?Sized,
    Phase: Ord + Default,
{
    /// Register a listener to this event for the default phase.
    pub fn register(&mut self, listener: Box<T>) {
        self.register_with_phase(listener, Default::default())
    }
}

impl<T, Phase> Drop for Event<T, Phase>
where
    T: ?Sized,
    Phase: Ord,
{
    fn drop(&mut self) {
        let mut vec = Vec::new();
        std::mem::swap(&mut self.listeners_and_cache.get_mut().0, &mut vec);

        for value in vec {
            let _ = unsafe { Box::from_raw(value.1 as *mut T) };
        }
    }
}

unsafe impl<T, Phase> Send for Event<T, Phase>
where
    T: ?Sized,
    Phase: Ord + Send,
{
}

unsafe impl<T, Phase> Sync for Event<T, Phase>
where
    T: ?Sized,
    Phase: Ord + Sync,
{
}

#[cfg(test)]
mod event_tests {
    use super::Event;

    #[test]
    fn registering_invoking() {
        let mut event: Event<dyn Fn(&str) -> bool> = Event::new(|listeners| {
            Box::new(move |string| {
                for listener in listeners {
                    if !listener(string) {
                        return false;
                    }
                }
                true
            })
        });

        assert!(event.invoker()(
            "minecraft by mojang is a propritary software."
        ));

        event.register(Box::new(|string| {
            !string.to_lowercase().contains("propritary software")
        }));
        event.register(Box::new(|string| !string.to_lowercase().contains("mojang")));
        event.register(Box::new(|string| {
            !string.to_lowercase().contains("minecraft")
        }));

        assert!(!event.invoker()(
            "minecraft by mojang is a propritary software."
        ));

        assert!(event.invoker()("i love krlite."));

        event.register(Box::new(|string| !string.to_lowercase().contains("krlite")));

        assert!(!event.invoker()("i love krlite."));
    }

    #[test]
    fn phases() {
        let mut event: Event<dyn Fn(&mut String)> = Event::new(|listeners| {
            Box::new(move |string| {
                for listener in listeners {
                    listener(string);
                }
            })
        });

        event.register(Box::new(|string| string.push_str("genshin impact ")));
        event.register_with_phase(Box::new(|string| string.push_str("you're right, ")), -3);
        event.register_with_phase(Box::new(|string| string.push_str("but ")), -2);
        event.register_with_phase(Box::new(|string| string.push_str("is a...")), 10);

        {
            let mut string = String::new();
            event.invoker()(&mut string);
            assert_eq!(string, "you're right, but genshin impact is a...");
        }

        event.register_with_phase(
            Box::new(|string| string.push_str("genshin impact, bootstrap! ")),
            -100,
        );

        {
            let mut string = String::new();
            event.invoker()(&mut string);
            assert_eq!(
                string,
                "genshin impact, bootstrap! you're right, but genshin impact is a..."
            );
        }
    }
}
