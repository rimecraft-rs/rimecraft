//! Per-test pool of resources.

use std::{collections::HashMap, ptr::NonNull};

use crate::TestId;

/// Pool of per-test resources.
#[derive(Debug)]
pub struct Pool<T> {
    resources: HashMap<TestId, NonNull<T>>,
}

impl<T> Pool<T> {
    /// Creates a new pool.
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Gets a resource for the current test, initializing it if necessary.
    #[allow(clippy::missing_panics_doc)]
    pub fn get_or_init<F>(&mut self, init: F) -> NonNull<T>
    where
        F: FnOnce() -> T,
    {
        let test = TestId::current();
        if let Some(resource) = self.resources.get(&test) {
            *resource
        } else {
            let resource = NonNull::new(Box::into_raw(Box::new(init()))).expect("allocate failed");
            self.resources.insert(test, resource);
            resource
        }
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::new()
    }
}
