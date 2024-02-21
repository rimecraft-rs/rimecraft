/// Iterator over the palette.
#[derive(Debug)]
pub struct Iter<'a, I, T> {
    pub(crate) internal: IterImpl<'a, I, T>,
}

#[derive(Debug)]
pub(crate) enum IterImpl<'a, I, T> {
    MaybeNone(std::option::Iter<'a, T>),
    Vector(std::slice::Iter<'a, T>),
    IntoIter(I),
}

impl<'a, I, T> Iterator for Iter<'a, I, T>
where
    I: Iterator<Item = &'a T>,
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.internal {
            IterImpl::MaybeNone(iter) => iter.next(),
            IterImpl::Vector(iter) => iter.next(),
            IterImpl::IntoIter(iter) => iter.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.internal {
            IterImpl::MaybeNone(iter) => iter.size_hint(),
            IterImpl::Vector(iter) => iter.size_hint(),
            IterImpl::IntoIter(iter) => iter.size_hint(),
        }
    }
}

impl<'a, I, T> ExactSizeIterator for Iter<'a, I, T>
where
    I: Iterator<Item = &'a T> + ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        match &self.internal {
            IterImpl::MaybeNone(iter) => iter.len(),
            IterImpl::Vector(iter) => iter.len(),
            IterImpl::IntoIter(iter) => iter.len(),
        }
    }
}
