#[cfg(test)]
mod tests;

use std::ops::Deref;

use parking_lot::{RwLock, RwLockReadGuard};

/// A type containing listeners of this event,
/// which can be invoked by an invoker.
///
/// The listeners are sorted by phases ([`i8`] by default)
/// that can be called in order.
pub struct Event<T, F, P> {
    listeners: Vec<(T, P)>,
    factory: F,
    invoker: RwLock<Option<T>>,
}

/// A sequence of listeners.
#[derive(Debug)]
pub struct Listeners<T> {
    empty: bool,
    inner: std::vec::IntoIter<T>,
}

impl<T> Listeners<T> {
    /// Whether there are no listeners
    /// in this sequence.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.empty
    }
}

impl<T: Deref> Iterator for Listeners<T> {
    type Item = Listener<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|val| Listener { inner: val })
    }
}

/// A cell that can be dereferenced
/// to the listener.
#[derive(Debug)]
pub struct Listener<T> {
    inner: T,
}

impl<T: Deref> Deref for Listener<T> {
    type Target = <T as Deref>::Target;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

#[derive(Debug)]
pub struct Invoker<'a, T> {
    inner: RwLockReadGuard<'a, Option<T>>,
}

impl<'a, T: Deref> Deref for Invoker<'a, T> {
    type Target = <T as Deref>::Target;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.inner.as_ref().unwrap()
    }
}

impl<T, F, P> Event<T, F, P> {
    /// Creates a new event with provided invoker factory.
    pub const fn new(factory: F) -> Self {
        Self {
            listeners: Vec::new(),
            factory,
            invoker: RwLock::new(None),
        }
    }

    #[inline]
    fn make_dirty(&mut self) {
        self.invoker.get_mut().take();
    }

    /// Registers a listener with given phase into
    /// this event.
    #[inline]
    pub fn register<L>(&mut self, listener: L, phase: P)
    where
        T: From<L>,
    {
        self.listeners.push((listener.into(), phase));
        self.make_dirty()
    }
}

impl<T, F, P> Event<T, F, P>
where
    F: Fn(Listeners<T>) -> T,
    P: Ord,
    T: Clone,
{
    /// Obtains the invoker of this event.
    pub fn invoker(&self) -> Invoker<'_, T> {
        {
            let rg = self.invoker.read();
            if rg.as_ref().is_some() {
                return Invoker { inner: rg };
            }
        }

        let mut listeners = self
            .listeners
            .iter()
            .map(|(l, p)| (l.clone(), p))
            .collect::<Vec<_>>();
        listeners.sort_unstable_by_key(|(_, p)| *p);
        let listeners = Listeners {
            empty: listeners.is_empty(),
            inner: listeners
                .into_iter()
                .map(|(l, _)| l)
                .collect::<Vec<_>>()
                .into_iter(),
        };

        *self.invoker.write() = Some((self.factory)(listeners));
        Invoker {
            inner: self.invoker.read(),
        }
    }
}
