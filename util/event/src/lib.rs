//! Rust implementation of the Fabric API Event system.
//!
//! # Examples
//!
//! ```
//! # use rimecraft_event::*;
//! # use std::sync::Arc;
//! let mut event: DefaultSyncEvent<dyn Fn(&mut String) + Send + Sync> = Event::new(|listeners| {
//! Arc::new(move |string| {
//!     for listener in &listeners {
//!         listener(string);
//!     }
//! })
//! });
//!
//! register!(event, Arc::new(|string| string.push_str("genshin impact ")));
//! register!(
//!     event,
//!     Arc::new(|string| string.push_str("you're right, ")),
//!     -3,
//! );
//! register!(event, Arc::new(|string| string.push_str("but ")), -2);
//! register!(event, Arc::new(|string| string.push_str("is a...")), 10);
//!
//! {
//!     let mut string = String::new();
//!     event.invoker()(&mut string);
//!     assert_eq!(string, "you're right, but genshin impact is a...");
//! }
//!
//! register!(
//!     event,
//!     Arc::new(|string| string.push_str("genshin impact, bootstrap! ")),
//!     -100,
//! );
//!
//! {
//!     let mut string = String::new();
//!     event.invoker()(&mut string);
//!     assert_eq!(
//!         string,
//!         "genshin impact, bootstrap! you're right, but genshin impact is a..."
//!     );
//! }
//! ```

use std::{ops::Deref, rc::Rc, sync::Arc};

use parking_lot::{RwLock, RwLockReadGuard};

/// A type containing listeners of this event,
/// which can be invoked by an invoker.
///
/// The listeners are sorted by phases,
/// that can be called in order.
#[derive(Debug)]
pub struct Event<T, F, P> {
    listeners: Vec<(T, P)>,
    factory: F,
    invoker: RwLock<Option<T>>,
}

/// A sequence of listeners.
#[derive(Debug)]
pub struct Listeners<T> {
    inner: Arc<Vec<T>>,
}

impl<T> Listeners<T> {
    /// Returns the number of listeners
    /// in this sequence.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether there are no listeners
    /// in this sequence.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<'a, T: Deref> IntoIterator for &'a Listeners<T> {
    type Item = &'a <T as Deref>::Target;

    type IntoIter = ListenersIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ListenersIter {
            inner: self.inner.iter(),
        }
    }
}

impl<T> Clone for Listeners<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// An iterator over listeners.
#[derive(Debug)]
pub struct ListenersIter<'a, T> {
    inner: std::slice::Iter<'a, T>,
}

impl<'a, T: Deref> Iterator for ListenersIter<'a, T> {
    type Item = &'a <T as Deref>::Target;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Deref::deref)
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
        &self.inner
    }
}

/// A guard that can be dereferenced
/// to the invoker.
#[derive(Debug)]
pub struct Invoker<'a, T> {
    inner: RwLockReadGuard<'a, Option<T>>,
}

impl<T: Deref> Deref for Invoker<'_, T> {
    type Target = <T as Deref>::Target;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
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
    pub fn register(&mut self, listener: T, phase: P) {
        self.listeners.push((listener, phase));
        self.make_dirty()
    }

    /// Returns the number of listeners.
    #[inline]
    pub fn len(&self) -> usize {
        self.listeners.len()
    }

    /// Returns whether there are no listeners.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.listeners.is_empty()
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
        listeners.sort_by_key(|(_, p)| *p);
        let listeners = Listeners {
            inner: Arc::new(listeners.into_iter().map(|(l, _)| l).collect()),
        };

        *self.invoker.write() = Some((self.factory)(listeners));
        Invoker {
            inner: self.invoker.read(),
        }
    }
}

/// Registers a listener into the event.
#[macro_export]
macro_rules! register {
    ($e:expr, $l:expr, $p:expr$(,)?) => {
        $e.register($l, $p)
    };
    ($e:expr, $l:expr$(,)?) => {
        $crate::register!($e, $l, ::core::default::Default::default())
    };
}

/// Defaulted invoker factory type.
pub type InvokerFactory<T> = fn(Listeners<T>) -> T;
/// Defaulted synced event type.
pub type DefaultSyncEvent<T, P = i8> = Event<Arc<T>, InvokerFactory<Arc<T>>, P>;
/// Defaulted unsynced event type.
pub type DefaultEvent<T, P = i8> = Event<Rc<T>, InvokerFactory<Rc<T>>, P>;

#[cfg(test)]
mod tests;
