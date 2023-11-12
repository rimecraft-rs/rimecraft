use std::{
    fmt::Debug,
    sync::{Arc, OnceLock},
};

use parking_lot::RwLock;
use rimecraft_primitives::combine_traits;

use crate::text::{OrderedText, Text};

pub const DEFAULT_LANG: &str = "en_us";

/// Represents a language.
pub trait Lang {
    /// Returns the translation of given translation key.
    fn translation<'a>(&'a self, key: &str) -> Option<&'a str>;

    /// Returns the direction of the language.
    #[inline]
    fn direction(&self) -> Direction {
        Default::default()
    }

    fn reorder<'a>(&self, text: &'a Text) -> Box<dyn OrderedText + Send + Sync + 'a>;
}

/// Extensions for types implemented [`Lang`].
pub trait LangExt: Lang {
    /// Returns the translation of given translation key or the key itself.
    #[inline]
    fn translation_or_key<'a>(&'a self, key: &'a str) -> &'a str {
        self.translation(key).unwrap_or(key)
    }
}

impl<T: Lang + ?Sized> LangExt for T {}

combine_traits! {
    pub(crate) trait DebugLang: Lang, Debug, Send, Sync
}

/// Direction of a language.
///
/// # Examples
///
/// English is [`Self::LeftToRight`],
/// and Arabic is [`Self::RightToLeft`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum Direction {
    /// Left to right.
    #[default]
    LeftToRight,
    /// Right to left.
    RightToLeft,
}

/// The global instance of language.
static INSTANCE: OnceLock<RwLock<Arc<dyn DebugLang>>> = OnceLock::new();

/// Gets the global language instance.
///
/// For initializing the instance, see [`set_global`].
pub fn global() -> Option<Global> {
    INSTANCE.get().map(|lock| Global { guard: lock.read() })
}

/// Sets the global language instance.
///
/// See [`global`] for getting the instance.
pub fn set_global<T: 'static>(instance: T)
where
    T: Lang + Debug + Send + Sync,
{
    let boxed = Arc::new(instance);
    if let Some(value) = INSTANCE.get() {
        *value.write() = boxed;
    } else {
        INSTANCE.set(RwLock::new(boxed)).unwrap()
    }
}

/// The global language instance returned by [`global`].
#[derive(Debug)]
pub struct Global {
    guard: parking_lot::RwLockReadGuard<'static, Arc<dyn DebugLang>>,
}

impl Lang for Global {
    #[inline]
    fn translation<'a>(&'a self, key: &str) -> Option<&'a str> {
        self.guard.translation(key)
    }

    #[inline]
    fn direction(&self) -> Direction {
        self.guard.direction()
    }

    #[inline]
    fn reorder<'a>(&self, text: &'a Text) -> Box<dyn OrderedText + Send + Sync + 'a> {
        self.guard.reorder(text)
    }
}
