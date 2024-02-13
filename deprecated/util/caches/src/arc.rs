use std::{
    collections::HashSet,
    hash::Hash,
    ops::Deref,
    sync::{Arc, Mutex, Weak},
};

/// A variant of hash-based [`crate::Caches`], where values are stored in weak
/// pointers and values are provided with [`Arc`].
///
/// Caches with zero strong count will be soon destroyed.
pub struct Caches<T>
where
    T: Hash + Eq + 'static,
{
    map: Mutex<HashSet<WeakNode<'static, T>>>,
}

impl<T> Caches<T>
where
    T: Hash + Eq + 'static,
{
    /// Creates a new caches.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Obtain an [`Arc`] from cached weak pointers in this caches,
    /// and the provided value will be dropped.
    /// If an equaled value doesn't exist in this caches, the value
    /// will be stored in a new [`Arc`].
    pub fn get(&self, value: T) -> Arc<T> {
        let mut guard = self.map.lock().unwrap();
        if let Some(v) = guard.get(&WeakNode::Ref(unsafe { &*(&value as *const T) })) {
            if let WeakNode::Stored(weak) = v {
                weak.upgrade().expect("invalid weak pointer")
            } else {
                unreachable!()
            }
        } else {
            let arc = Arc::new(value);
            guard.insert(WeakNode::Stored(Arc::downgrade(&arc)));
            arc
        }
    }

    /// Whether this caches contains the value.
    #[inline]
    pub fn contains(&self, value: &T) -> bool {
        self.map
            .lock()
            .unwrap()
            .contains(&WeakNode::Ref(unsafe { &*(value as *const T) }))
    }
}

impl<T> Default for Caches<T>
where
    T: Hash + Eq,
{
    #[inline]
    fn default() -> Self {
        Self {
            map: Mutex::new(HashSet::new()),
        }
    }
}

enum WeakNode<'a, T> {
    Stored(Weak<T>),
    Ref(&'a T),
}

impl<T> Hash for WeakNode<'_, T>
where
    T: Hash,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            WeakNode::Stored(value) => {
                if let Some(v) = value.upgrade() {
                    v.hash(state)
                }
            }
            WeakNode::Ref(value) => value.hash(state),
        }
    }
}

impl<T> PartialEq for WeakNode<'_, T>
where
    T: Hash + Eq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (WeakNode::Stored(value0), WeakNode::Stored(value1)) => value0.ptr_eq(value1),
            (WeakNode::Ref(value0), WeakNode::Stored(value1)) => {
                value1.upgrade().map_or(false, |e| e.deref() == *value0)
            }
            (WeakNode::Stored(value0), WeakNode::Ref(value1)) => {
                value0.upgrade().map_or(false, |e| e.deref() == *value1)
            }
            (WeakNode::Ref(value0), WeakNode::Ref(value1)) => value0 == value1,
        }
    }
}

impl<T> Eq for WeakNode<'_, T> where T: Hash + Eq {}
