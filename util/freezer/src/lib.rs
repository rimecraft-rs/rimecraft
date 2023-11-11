use std::{
    ops::{Deref, DerefMut},
    sync::OnceLock,
    sync::{Mutex, MutexGuard},
};

/// A type that contains either mutable value or immutable value,
/// where the mutable one can be freezed into the immutable one.
pub struct Freezer<I, M = I> {
    immutable: OnceLock<I>,
    mutable: Mutex<Option<M>>,
}

impl<I, M> Freezer<I, M>
where
    M: Freeze<I>,
{
    /// Creates a new freezer.
    #[inline]
    pub const fn new(mutable: M) -> Self {
        Self {
            immutable: OnceLock::new(),
            mutable: Mutex::new(Some(mutable)),
        }
    }

    /// Freeze this instance with provided options.
    pub fn freeze(&self, opts: M::Opts) {
        assert!(!self.is_freezed(), "cannot freeze a freezed freezer");

        let _ = self
            .immutable
            .set(self.mutable.lock().unwrap().take().unwrap().freeze(opts));
    }

    /// Whether this instance has been already freezed.
    #[inline]
    pub fn is_freezed(&self) -> bool {
        self.immutable.get().is_some()
    }

    /// Locks the mutable value and returns a guard of it.
    #[inline]
    pub fn lock(&self) -> Guard<'_, M> {
        Guard {
            inner: self.mutable.lock().unwrap(),
        }
    }

    /// Gets the immutable instance.
    #[inline]
    pub fn get(&self) -> Option<&I> {
        self.immutable.get()
    }
}

impl<I, M> Freezer<I, M>
where
    M: Freeze<I>,
    M::Opts: Default,
{
    /// Gets the immutable instance, or initialize it with
    /// a default value.
    pub fn get_or_freeze(&self) -> &I {
        if !self.is_freezed() {
            self.freeze(Default::default());
        }

        self.immutable.get().unwrap()
    }
}

/// A mutex guard wrapper.
pub struct Guard<'a, T: 'a> {
    inner: MutexGuard<'a, Option<T>>,
}

impl<'a, T: 'a> Deref for Guard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<'a, T: 'a> DerefMut for Guard<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

/// Describes a type that can be used for mutable instance (`M`)
/// in a [`Freezer`].
///
/// The generic type `T` is the freeze output type of this type.
///
/// By default, all types will can be freezed into themselves
/// with empty tuple options.
pub trait Freeze<T> {
    /// Options for the freeze operation.
    type Opts;

    /// Build and freeze this value into `T` with options.
    fn freeze(self, opts: Self::Opts) -> T;
}

impl<T> Freeze<T> for T {
    type Opts = ();

    #[inline]
    fn freeze(self, _opts: Self::Opts) -> T {
        self
    }
}
