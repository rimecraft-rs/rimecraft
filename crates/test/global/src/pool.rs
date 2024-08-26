//! Per-test pool of resources.

use std::{collections::HashMap, ptr::NonNull};

use parking_lot::Mutex;

use crate::TestId;

/// Pool of per-test resources.
#[derive(Debug)]
pub struct Pool<T> {
    resources: Mutex<HashMap<TestId, NonNull<T>>>,
}

impl<T> Pool<T> {
    /// Creates a new pool.
    pub fn new() -> Self {
        Self {
            resources: Mutex::new(HashMap::new()),
        }
    }

    /// Gets a resource for the current test, initializing it if necessary.
    ///
    /// The returned pointer is valid for the lifetime of the pool.
    #[allow(clippy::missing_panics_doc)]
    pub fn get_or_init<F>(&self, init: F) -> *const T
    where
        F: FnOnce() -> T,
    {
        let mut resources = self.resources.lock();
        let test = TestId::current();
        if let Some(resource) = resources.get(&test) {
            *resource
        } else {
            let resource = NonNull::new(Box::into_raw(Box::new(init()))).expect("OOM");
            resources.insert(test, resource);
            resource
        }
        .as_ptr()
        .cast_const()
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Pool<T> {
    fn drop(&mut self) {
        let resources = self.resources.get_mut();
        for resource in resources.values() {
            drop(unsafe { Box::from_raw(resource.as_ptr()) })
        }
    }
}

unsafe impl<T: Send> Send for Pool<T> {}
unsafe impl<T: Sync> Sync for Pool<T> {}
