use std::{
    fmt::Debug,
    ops::Deref,
    sync::{Arc, OnceLock, Weak},
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
    pub trait DebugLang: Lang, Debug, Send, Sync
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

/// A value that depends on the global language instance.
/// When the global language instance has been changed,
/// the inner value will be updated.
///
/// See [`UpdateLang`].
#[derive(Debug)]
pub struct LangDepended<T> {
    inner: RwLock<LangDependedInner<T>>,
}

impl<T> LangDepended<T> {
    /// Creates a new [`LangDepended`] with given inner value.
    ///
    /// This action does not acquire the global language instance.
    pub fn new(inner: T) -> Self {
        Self {
            inner: RwLock::new(LangDependedInner { lang: None, inner }),
        }
    }
}

impl<T: UpdateLang> LangDepended<T> {
    /// Gets the inner value.
    ///
    /// When the global language instance has been changed,
    /// the inner value will be updated.
    #[inline]
    pub fn get(&self, cx: &T::Context) -> LangDependedRef<'_, T> {
        self.update(cx);
        LangDependedRef {
            inner: self.inner.read(),
        }
    }

    fn update(&self, cx: &T::Context) {
        let lang_arc = INSTANCE.get().unwrap().read().clone();
        let lang = Arc::downgrade(&lang_arc);

        if !self
            .inner
            .read()
            .lang
            .as_ref()
            .is_some_and(|e| e.ptr_eq(&lang))
        {
            let mut inner = self.inner.write();
            inner.lang = Some(lang);
            inner.inner.update_from_lang(&*lang_arc, cx);
        }
    }

    /// Gets the mutable inner value.
    ///
    /// When the global language instance has been changed,
    /// the inner value will be updated.
    #[inline]
    pub fn get_mut(&mut self, cx: &T::Context) -> &mut T {
        self.update_mut(cx);
        &mut self.inner.get_mut().inner
    }

    fn update_mut(&mut self, cx: &T::Context) {
        let lang_arc = INSTANCE.get().unwrap().read().clone();
        let lang = Arc::downgrade(&lang_arc);
        let inner = self.inner.get_mut();

        if !inner.lang.as_ref().is_some_and(|e| e.ptr_eq(&lang)) {
            inner.lang = Some(lang);
            inner.inner.update_from_lang(&*lang_arc, cx);
        }
    }
}

impl<T> Clone for LangDepended<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: RwLock::new(self.inner.read().clone()),
        }
    }
}

/// Represents types that can be updated after the change of
/// global language instance.
///
/// See [`LangDepended`].
pub trait UpdateLang {
    type Context: ?Sized;

    /// Updates this value from given language.
    fn update_from_lang(&mut self, lang: &dyn DebugLang, cx: &Self::Context);
}

#[derive(Debug, Clone)]
struct LangDependedInner<T> {
    lang: Option<Weak<dyn DebugLang>>,
    inner: T,
}

/// Read RAII guard of [`LangDepended`], represented as
/// a reference.
///
/// See [`LangDepended::get`].
pub struct LangDependedRef<'a, T> {
    inner: parking_lot::RwLockReadGuard<'a, LangDependedInner<T>>,
}

impl<T> AsRef<T> for LangDependedRef<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.inner.inner
    }
}

impl<T> Deref for LangDependedRef<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner.inner
    }
}
