//! Utilities for freezing a mutable value into immutable value.
//!
//! # Examples
//!
//! ```
//! # use rimecraft_freezer::Freezer;
//! let freezer: Freezer<[i32; 3]> = Freezer::new([1, 2, 3]);
//!
//! assert_eq!(freezer.get(), None);
//! freezer.lock().unwrap()[1] = 3;
//!
//! assert_eq!(freezer.get_or_freeze(), &[1, 3, 3]);
//! assert!(freezer.lock().is_none());
//! ```

use std::{
    ops::{Deref, DerefMut},
    sync::OnceLock,
    sync::{Mutex, MutexGuard},
};

/// A cell that contains either mutable value or immutable value,
/// where the mutable one can be freezed into the immutable one.
#[derive(Debug)]
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
    ///
    /// # Panics
    ///
    /// This function will panic if this instance
    /// has been already freezed.
    pub fn freeze(&self, opts: M::Opts) {
        assert!(!self.is_freezed(), "cannot freeze a freezed freezer");

        let _result = self
            .immutable
            .set(self.mutable.lock().unwrap().take().unwrap().freeze(opts));
    }

    /// Whether this instance has been already freezed.
    #[inline]
    pub fn is_freezed(&self) -> bool {
        self.immutable.get().is_some()
    }

    /// Locks the mutable value and returns a guard of it.
    ///
    /// Returns `None` if this instance has been already freezed.
    ///
    /// # Panics
    ///
    /// This function will panic if the lock is held by
    /// another user and the user panics when holding the lock.
    #[inline]
    pub fn lock(&self) -> Option<Guard<'_, M>> {
        if self.is_freezed() {
            None
        } else {
            Some(Guard {
                inner: self.mutable.lock().unwrap(),
            })
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
    ///
    /// # Panics
    ///
    /// Panics if the freeze operation was failed.
    pub fn get_or_freeze(&self) -> &I {
        if !self.is_freezed() {
            self.freeze(Default::default());
        }

        self.immutable.get().unwrap()
    }
}

/// A mutex guard wrapper.
#[derive(Debug)]
pub struct Guard<'a, T: 'a> {
    inner: MutexGuard<'a, Option<T>>,
}

impl<'a, T: 'a> Deref for Guard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect("freezer already freezed")
    }
}

impl<'a, T: 'a> DerefMut for Guard<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().expect("freezer already freezed")
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

impl<T, U> Freeze<U> for T
where
    T: Into<U>,
{
    type Opts = ();

    #[inline]
    fn freeze(self, _opts: Self::Opts) -> U {
        self.into()
    }
}
