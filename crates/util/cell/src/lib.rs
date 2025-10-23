//! Cell abstraction for Rimecraft.

/// A generic cell abstraction that can provide interior mutability.
pub trait Cell<T>
where
    T: ?Sized,
{
    /// The read guard type returned by [`Cell::read`].
    type ReadGuard<'a>: std::ops::Deref<Target = T> + 'a
    where
        Self: 'a;

    /// The write guard type returned by [`Cell::write`].
    type WriteGuard<'a>: std::ops::DerefMut<Target = T> + 'a
    where
        Self: 'a;

    /// Reads the value, returning a read guard.
    fn read(&self) -> Self::ReadGuard<'_>;

    /// Writes to the value, returning a write guard.
    fn write(&self) -> Self::WriteGuard<'_>;
}

#[cfg(feature = "refcell")]
impl<T> Cell<T> for std::cell::RefCell<T> {
    type ReadGuard<'a>
        = std::cell::Ref<'a, T>
    where
        Self: 'a;
    type WriteGuard<'a>
        = std::cell::RefMut<'a, T>
    where
        Self: 'a;

    fn read(&self) -> Self::ReadGuard<'_> {
        self.borrow()
    }

    fn write(&self) -> Self::WriteGuard<'_> {
        self.borrow_mut()
    }
}

#[cfg(feature = "rwlock")]
impl<T> Cell<T> for std::sync::RwLock<T> {
    type ReadGuard<'a>
        = std::sync::RwLockReadGuard<'a, T>
    where
        Self: 'a;

    type WriteGuard<'a>
        = std::sync::RwLockWriteGuard<'a, T>
    where
        Self: 'a;

    fn read(&self) -> Self::ReadGuard<'_> {
        self.read().unwrap()
    }

    fn write(&self) -> Self::WriteGuard<'_> {
        self.write().unwrap()
    }
}
