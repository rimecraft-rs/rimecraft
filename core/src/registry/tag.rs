use rimecraft_primitives::Id;

static KEYS_CACHE: once_cell::sync::Lazy<rimecraft_caches::ArcCaches<Id>> =
    once_cell::sync::Lazy::new(rimecraft_caches::ArcCaches::new);

/// Represents a tag key.
pub struct Key<T> {
    reg: super::Key<super::Registry<T>>,
    id: std::sync::Arc<Id>,
}

impl<T> Key<T> {
    #[inline]
    pub fn new(reg: super::Key<super::Registry<T>>, id: Id) -> Self {
        Self {
            reg,
            id: KEYS_CACHE.get(id),
        }
    }

    #[inline]
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

    #[inline]
    pub fn reg(&self) -> super::Key<super::Registry<T>> {
        self.reg
    }

    #[inline]
    pub fn id(&self) -> &Id {
        &self.id
    }
}

impl<T> Clone for Key<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            reg: self.reg,
            id: self.id.clone(),
        }
    }
}

impl<T> PartialEq for Key<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.reg == other.reg
    }
}

impl<T> Eq for Key<T> {}

impl<T> std::fmt::Debug for Key<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TagKey[{:?} / {}]", self.reg, self.id)
    }
}

impl<T> std::hash::Hash for Key<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
