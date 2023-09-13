use crate::prelude::*;

static KEYS_CACHE: once_cell::sync::Lazy<crate::collections::ArcCaches<Id>> =
    once_cell::sync::Lazy::new(crate::collections::ArcCaches::new);

/// Represents a tag key.
pub struct Key<T> {
    reg: super::Key<super::Registry<T>>,
    id: std::sync::Arc<Id>,
}

impl<T> Key<T> {
    pub fn new(reg: super::Key<super::Registry<T>>, id: Id) -> Self {
        Self {
            reg,
            id: KEYS_CACHE.get(id),
        }
    }

    pub fn is_of<T1>(&self, reg: &super::Key<super::Registry<T1>>) -> bool {
        self.reg.inner == reg.inner
    }

    /// Return `Some` if the key is of reg, otherwise `None`.
    pub fn cast<E>(&self, reg: &super::Key<super::Registry<E>>) -> Option<Key<E>> {
        if self.is_of(reg) {
            Some(Key {
                reg: super::Key {
                    _type: std::marker::PhantomData,
                    inner: self.reg.inner,
                },
                id: self.id.clone(),
            })
        } else {
            None
        }
    }

    pub fn reg(&self) -> super::Key<super::Registry<T>> {
        self.reg
    }

    pub fn id(&self) -> &Id {
        &self.id
    }
}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        Self {
            reg: self.reg,
            id: self.id.clone(),
        }
    }
}

impl<T> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.reg == other.reg
    }
}

impl<T> Eq for Key<T> {}

impl<T> std::fmt::Debug for Key<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TagKey[")?;
        self.reg.fmt(f)?;
        f.write_str(" / ")?;
        std::fmt::Display::fmt(&self.id, f)?;
        f.write_str("]")
    }
}

impl<T> std::hash::Hash for Key<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
