#[cfg(test)]
mod tests;

use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::RwLock;

/// Listeners and cache.
type LisCac<T, Phase> = (Vec<(Phase, *const T)>, Option<Box<T>>, Vec<&'static T>);

/// A type containing listeners of this event,
/// which can be invoked by an invoker.
///
/// The listeners are sorted by phases ([`i8`] by default)
/// that can be called in order.
pub struct Event<T, Phase = i8>
where
    T: ?Sized + 'static,
{
    /// Whether listeners has been modified before requesting the invoker.
    dirty: AtomicBool,

    invoker_factory: fn(&'static [&'static T]) -> Box<T>,

    /// 0: raw listeners with phases\
    /// 1: cached invoker\
    /// 2: cached listener references
    lis_cac: RwLock<LisCac<T, Phase>>,
}

impl<T, Phase> Event<T, Phase>
where
    T: ?Sized,
    Phase: Ord,
{
    /// Create a new event with provided event factory.
    ///
    /// To avoid lifetime problems in the factory, listeners
    /// provided are all in static references so that they're
    /// able to be copied and moved.
    /// So you should add a `move` keyword before the closure
    /// to return in the factory.
    pub const fn new(factory: fn(&'static [&'static T]) -> Box<T>) -> Self {
        Self {
            lis_cac: RwLock::new((Vec::new(), None, Vec::new())),
            invoker_factory: factory,
            dirty: AtomicBool::new(false),
        }
    }

    /// Get the invoker of this event.
    ///
    /// Once the invoker is created, it will be cached until
    /// the next modification of listeners, and will be re-created
    /// by the factory.
    pub fn invoker(&self) -> &T {
        if self.dirty.load(Ordering::Acquire) {
            let mut write_guard = self.lis_cac.write();
            write_guard.0.sort_by(|e0, e1| Phase::cmp(&e0.0, &e1.0));
            self.dirty.store(false, Ordering::Release);

            write_guard.2 = write_guard.0.iter().map(|e| unsafe { &*e.1 }).collect();
            write_guard.1 = Some((self.invoker_factory)(unsafe {
                &*(&write_guard.2 as *const Vec<&'static T>)
            }));
        } else if self.lis_cac.read().1.is_none() {
            let mut write_guard = self.lis_cac.write();
            write_guard.1 = Some((self.invoker_factory)(unsafe {
                &*(&write_guard.2 as *const Vec<&'static T>)
            }));
        }

        unsafe { &*(&**self.lis_cac.read().1.as_ref().unwrap() as *const T) }
    }

    /// Register a listener to this event for the specified phase.
    pub fn register_with_phase(&mut self, listener: Box<T>, phase: Phase) {
        self.lis_cac
            .get_mut()
            .0
            .push((phase, Box::into_raw(listener)));

        if !self.dirty.load(Ordering::Acquire) {
            self.dirty.store(true, Ordering::Release);
        }
    }
}

impl<T, Phase> Event<T, Phase>
where
    T: ?Sized,
    Phase: Ord + Default,
{
    /// Register a listener to this event for the default phase.
    #[inline]
    pub fn register(&mut self, listener: Box<T>) {
        self.register_with_phase(listener, Default::default())
    }
}

impl<T, Phase> Drop for Event<T, Phase>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        let mut vec = Vec::new();
        std::mem::swap(&mut self.lis_cac.get_mut().0, &mut vec);

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
