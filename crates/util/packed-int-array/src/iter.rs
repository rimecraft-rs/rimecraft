use crate::PackedIntArray;

#[derive(Debug)]
pub(crate) struct IterInner {
    pub l: u64,
    pub j: usize,
    pub times: usize,
}

/// An iterator over a packed int array.
#[derive(Debug)]
pub struct Iter<'a> {
    pub(crate) array: &'a PackedIntArray,
    pub(crate) iter: std::slice::Iter<'a, u64>,
    pub(crate) inner: IterInner,
}

impl Iterator for Iter<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.times >= self.array.len() {
            return None;
        }

        if self.inner.j < self.array.elements_per_long {
            self.inner.j += 1;
            let res = self.inner.l & self.array.max;
            self.inner.l >>= self.array.element_bits;
            self.inner.times += 1;
            Some(res as u32)
        } else {
            self.inner.l = *self.iter.next()?;
            self.next()
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.array.len() - self.inner.times;
        (len, Some(len))
    }
}

impl ExactSizeIterator for Iter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.array.len() - self.inner.times
    }
}

/// An iterator over a packed int array.
#[derive(Debug)]
pub struct IntoIter {
    pub(crate) element_bits: usize,
    pub(crate) elements_per_long: usize,
    pub(crate) max: u64,
    pub(crate) iter: std::vec::IntoIter<u64>,
    pub(crate) inner: IterInner,
    pub(crate) len: usize,
}

impl Iterator for IntoIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.times >= self.len {
            return None;
        }

        if self.inner.j < self.elements_per_long {
            self.inner.j += 1;
            let res = self.inner.l & self.max;
            self.inner.l >>= self.element_bits;
            self.inner.times += 1;
            Some(res as u32)
        } else {
            self.inner.l = self.iter.next()?;
            self.next()
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len - self.inner.times;
        (len, Some(len))
    }
}

impl ExactSizeIterator for IntoIter {
    #[inline]
    fn len(&self) -> usize {
        self.len - self.inner.times
    }
}
