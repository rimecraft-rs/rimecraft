#![forbid(unsafe_code, missing_docs)]

//! Safe reference-counted interners.
//!
//! # Examples
//!
//! ```
//! # use rimecraft_interner::Interner;
//! let interner: Interner<'static, str> = Interner::new();
//!
//! interner.obtain("wow");
//! interner.obtain("mom");
//!
//! assert_eq!(&*interner.obtain("wow"), "wow");
//! assert_eq!(&*interner.obtain("mom"), "mom");
//! ```

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
    inner: DashSet<Node<'a, T>>,
}

#[derive(Debug)]
struct Node<'a, T: ?Sized>(Ref<'a, T>);

impl<T: ?Sized> Hash for Node<'_, T>
where
    T: Hash,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: ?Sized> PartialEq for Node<'_, T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T: ?Sized> Eq for Node<'_, T> where T: Eq {}

#[allow(single_use_lifetimes)]
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

impl<T: ?Sized> Hash for Ref<'_, T>
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

impl<T: ?Sized> PartialEq for Ref<'_, T>
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

impl<T: ?Sized> Eq for Ref<'_, T> where T: Eq {}

impl<T: ?Sized> Interner<'_, T>
where
    T: Hash + Eq,
{
    /// Creates a new interner.
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: DashSet::new(),
        }
    }

    /// Shrinks the interner's internal storage.
    pub fn shrink(&self) {
        self.inner.retain(|node| {
            let Node(Ref::Weak(weak)) = node else {
                unreachable!()
            };
            weak.strong_count() > 0
        });
        self.inner.shrink_to_fit()
    }

    /// Gets or interns a value, returning a reference
    /// to the interned value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rimecraft_interner::Interner;
    /// let interner: Interner<'static, str> = Interner::new();
    /// assert_eq!(&*interner.obtain("ferris"), "ferris");
    /// ```
    pub fn obtain<Q>(&self, value: Q) -> Arc<T>
    where
        Q: Deref<Target = T> + IntoBoxed<T>,
    {
        if let Some(v) = self.inner.get(&Ref::Strong(&*value)) {
            let Node(Ref::Weak(ref weak)) = *v else {
                unreachable!()
            };
            if let Some(arc) = weak.upgrade() {
                arc
            } else {
                unreachable!()
            }
        } else {
            let value = value.into_boxed().into();
            self.inner.insert(Node(Ref::Weak(Arc::downgrade(&value))));
            value
        }
    }
}

impl<T: ?Sized> Interner<'_, T> where T: Hash + Eq {}

impl<T: ?Sized> Default for Interner<'_, T>
where
    T: Hash + Eq,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Debug for Interner<'_, T>
where
    T: Hash + Eq + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interner")
            .field("map", &self.inner)
            .finish()
    }
}

/// A trait for types that can be
/// converted into a box.
pub trait IntoBoxed<T: ?Sized> {
    /// Converts `self` into a boxed value.
    fn into_boxed(self) -> Box<T>;
}

impl<T, S> IntoBoxed<T> for &S
where
    S: ToOwned<Owned = T>,
{
    #[inline]
    fn into_boxed(self) -> Box<T> {
        Box::new(self.to_owned())
    }
}

impl IntoBoxed<str> for &str {
    #[inline]
    fn into_boxed(self) -> Box<str> {
        self.into()
    }
}

#[cfg(test)]
mod tests;
