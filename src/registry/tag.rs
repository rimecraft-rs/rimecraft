use crate::prelude::*;

/// Represents a tag key.
pub struct TagKey<T>(std::sync::Arc<(super::Key<super::Registry<T>>, Identifier)>);

impl<T> TagKey<T> {
    pub fn new(reg: super::Key<super::Registry<T>>, id: Identifier) -> Self {
        Self(std::sync::Arc::new((reg, id)))
    }

    pub fn is_of<T1>(&self, reg: &super::Key<super::Registry<T1>>) -> bool {
        self.0 .0.inner == reg.inner
    }

    /// Return `Some(_)` if the key is of reg, otherwise `None`.
    pub fn cast<E>(&self, reg: &super::Key<super::Registry<E>>) -> Option<TagKey<E>> {
        if self.is_of(reg) {
            Some(TagKey(std::sync::Arc::new((
                super::Key {
                    _type: std::marker::PhantomData,
                    inner: self.0 .0.inner.clone(),
                },
                self.0 .1.clone(),
            ))))
        } else {
            None
        }
    }

    pub fn reg(&self) -> &super::Key<super::Registry<T>> {
        &self.0 .0
    }

    pub fn id(&self) -> &Identifier {
        &self.0 .1
    }
}

impl<T> Clone for TagKey<T> {
    fn clone(&self) -> Self {
        Self(std::sync::Arc::clone(&self.0))
    }
}

impl<T> PartialEq for TagKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for TagKey<T> {}

impl<T> std::fmt::Display for TagKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TagKey[")?;
        self.0 .0.fmt(f)?;
        f.write_str(" / ")?;
        self.0 .1.fmt(f)?;
        f.write_str("]")
    }
}

impl<T> std::hash::Hash for TagKey<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}
