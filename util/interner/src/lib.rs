#![forbid(unsafe_code)]

use std::{
    borrow::Borrow,
    fmt::Debug,
    hash::Hash,
    ops::Deref,
    sync::{Arc, Weak},
};

use dashmap::DashSet;

/// A reference-counted interner.
pub struct Interner<'a, T: ?Sized> {
    map: DashSet<Node<'a, T>>,
}

#[derive(Debug)]
struct Node<'a, T: ?Sized>(Ref<'a, T>);

impl<'a, T: ?Sized> Hash for Node<'a, T>
where
    T: Hash,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<'a, T: ?Sized> PartialEq for Node<'a, T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<'a, T: ?Sized> Eq for Node<'a, T> where T: Eq {}

impl<'a: 'b, 'b, T: ?Sized> Borrow<Ref<'b, T>> for Node<'a, T> {
    #[inline]
    fn borrow(&self) -> &Ref<'b, T> {
        &self.0
    }
}

#[derive(Debug)]
enum Ref<'a, T: ?Sized> {
    Weak(Weak<T>),
    Strong(&'a T),
}

impl<'a, T: ?Sized> Hash for Ref<'a, T>
where
    T: Hash,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Ref::Weak(w) => {
                if let Some(s) = w.upgrade() {
                    <Option<&T>>::hash(&Some(&s), state)
                } else {
                    None::<&T>.hash(state)
                }
            }
            Ref::Strong(s) => Some(*s).hash(state),
        }
    }
}

impl<'a, T: ?Sized> PartialEq for Ref<'a, T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Ref::Weak(w1), Ref::Weak(w2)) => match (w1.upgrade(), w2.upgrade()) {
                (Some(s1), Some(s2)) => *s1 == *s2,
                (None, None) => true,
                _ => false,
            },
            (Ref::Strong(s1), Ref::Strong(s2)) => s1 == s2,
            (Ref::Strong(s1), Ref::Weak(w2)) => w2.upgrade().map_or(false, |s2| *s2 == **s1),
            (Ref::Weak(w1), Ref::Strong(s2)) => w1.upgrade().map_or(false, |s1| *s1 == **s2),
        }
    }
}

impl<'a, T: ?Sized> Eq for Ref<'a, T> where T: Eq {}

impl<'a, T: ?Sized> Interner<'a, T>
where
    T: Hash + Eq,
{
    /// Creates a new interner.
    #[inline]
    pub fn new() -> Self {
        Self {
            map: DashSet::new(),
        }
    }

    /// Shrinks the interner's internal storage.
    pub fn shrink(&self) {
        self.map.retain(|node| {
            let Node(Ref::Weak(weak)) = node else {
                unreachable!()
            };
            weak.strong_count() > 0
        });
        self.map.shrink_to_fit()
    }

    /// Gets or interns a value, returning a reference
    /// to the interned value.
    pub fn obtain<Q>(&self, value: Q) -> Arc<T>
    where
        Q: Deref<Target = T> + IntoBoxed<T>,
    {
        if let Some(v) = self.map.get(&Ref::Strong(&*value)) {
            let Node(Ref::Weak(ref weak)) = *v else {
                unreachable!()
            };
            weak.upgrade().unwrap().clone()
        } else {
            self.insert(value.into_boxed().into())
        }
    }

    #[inline]
    fn insert(&self, value: Arc<T>) -> Arc<T> {
        self.map.insert(Node(Ref::Weak(Arc::downgrade(&value))));
        value
    }
}

impl<'a, T: ?Sized> Interner<'a, T> where T: Hash + Eq {}

impl<'a, T: ?Sized> Default for Interner<'a, T>
where
    T: Hash + Eq,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: ?Sized> std::fmt::Debug for Interner<'a, T>
where
    T: Hash + Eq + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interner").field("map", &self.map).finish()
    }
}

/// A trait for types that can be
/// converted into a box.
pub trait IntoBoxed<T: ?Sized> {
    /// Converts `self` into a boxed value.
    fn into_boxed(self) -> Box<T>;
}

impl<'a, T, S> IntoBoxed<T> for &'a S
where
    S: ToOwned<Owned = T>,
{
    #[inline]
    fn into_boxed(self) -> Box<T> {
        Box::new(self.to_owned())
    }
}

impl<'a> IntoBoxed<str> for &'a str {
    #[inline]
    fn into_boxed(self) -> Box<str> {
        self.into()
    }
}

#[cfg(test)]
mod tests;
