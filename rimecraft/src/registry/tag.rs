use crate::prelude::*;

/// Represents a tag key.
pub struct TagKey<T> {
    inner: std::sync::Arc<(super::RegistryKey<super::Registry<T>>, Identifier)>,
}

impl<T> TagKey<T> {
    pub fn new(reg: super::RegistryKey<super::Registry<T>>, id: Identifier) -> Self {
        Self {
            inner: std::sync::Arc::new((reg, id)),
        }
    }

    pub fn is_of<T1>(&self, reg: &super::RegistryKey<super::Registry<T1>>) -> bool {
        self.inner.0.inner == reg.inner
    }

    pub fn cast<E>(&self, reg: &super::RegistryKey<super::Registry<E>>) -> Option<TagKey<E>> {
        if self.is_of(reg) {
            Some(TagKey {
                inner: std::sync::Arc::new((
                    super::RegistryKey {
                        _type: std::marker::PhantomData,
                        inner: self.inner.0.inner.clone(),
                    },
                    self.inner.1.clone(),
                )),
            })
        } else {
            None
        }
    }

    pub fn reg(&self) -> &super::RegistryKey<super::Registry<T>> {
        &self.inner.0
    }

    pub fn id(&self) -> &Identifier {
        &self.inner.1
    }
}

impl<T> Clone for TagKey<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> PartialEq for TagKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for TagKey<T> {}

impl<T> std::fmt::Display for TagKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TagKey[")?;
        self.inner.0.fmt(f)?;
        f.write_str(" / ")?;
        self.inner.1.fmt(f)?;
        f.write_str("]")
    }
}

impl<T> std::hash::Hash for TagKey<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}
