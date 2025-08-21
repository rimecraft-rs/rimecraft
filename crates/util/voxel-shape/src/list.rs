use std::{
    fmt::Debug,
    ops::{Add, Deref},
};

pub trait List<T> {
    fn index(&self, index: usize) -> T;
    fn iter(&self) -> impl IntoIterator<Item = T>;
    fn len(&self) -> usize;
}

pub trait ErasedList<T>: Send + Sync + Debug {
    fn __erased_index(&self, index: usize) -> T;

    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = T> + '_)) + '_));

    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a>
    where
        T: 'a;

    fn __erased_len(&self) -> usize;
}

impl<T> List<T> for [T]
where
    T: Copy,
{
    #[inline]
    fn index(&self, index: usize) -> T {
        self[index]
    }

    #[inline]
    fn iter(&self) -> impl IntoIterator<Item = T> {
        <[T]>::iter(self).copied()
    }

    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T, const N: usize> List<T> for [T; N]
where
    T: Copy,
{
    #[inline]
    fn index(&self, index: usize) -> T {
        self[index]
    }

    #[inline]
    fn iter(&self) -> impl IntoIterator<Item = T> {
        self.as_slice().iter().copied()
    }

    #[inline]
    fn len(&self) -> usize {
        N
    }
}

impl<L, T> List<T> for &L
where
    L: List<T> + ?Sized,
{
    #[inline]
    fn index(&self, index: usize) -> T {
        (**self).index(index)
    }

    #[inline]
    fn iter(&self) -> impl IntoIterator<Item = T> {
        (**self).iter()
    }

    #[inline]
    fn len(&self) -> usize {
        (**self).len()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FractionalDoubleList {
    pub section_count: usize,
}

impl List<f64> for FractionalDoubleList {
    #[inline]
    fn index(&self, index: usize) -> f64 {
        index as f64 / self.section_count as f64
    }

    fn iter(&self) -> impl IntoIterator<Item = f64> {
        (0..=self.section_count).map(|i| i as f64 / self.section_count as f64)
    }

    fn len(&self) -> usize {
        self.section_count + 1
    }
}

#[repr(transparent)]
pub struct ListDeref<T>(pub T);

impl<T, I> ErasedList<I> for ListDeref<T>
where
    T: Deref<Target: ErasedList<I>> + Send + Sync + Debug,
{
    #[inline]
    fn __erased_index(&self, index: usize) -> I {
        self.0.__erased_index(index)
    }

    #[inline]
    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = I> + '_)) + '_)) {
        self.0.__peek_erased_iter(f);
    }

    #[inline]
    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = I> + 'a>
    where
        I: 'a,
    {
        self.0.__boxed_erased_iter()
    }

    #[inline]
    fn __erased_len(&self) -> usize {
        self.0.__erased_len()
    }
}

#[repr(transparent)]
pub struct ListEraser<T: ?Sized>(pub T);

impl<L, T> ErasedList<T> for ListEraser<L>
where
    L: List<T> + Send + Sync + Debug + ?Sized,
{
    #[inline]
    fn __erased_index(&self, index: usize) -> T {
        self.0.index(index)
    }

    #[inline]
    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = T> + '_)) + '_)) {
        f(&mut self.0.iter().into_iter())
    }

    #[inline]
    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a>
    where
        T: 'a,
    {
        Box::new(self.0.iter().into_iter())
    }

    #[inline]
    fn __erased_len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Debug + ?Sized> Debug for ListEraser<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Debug> Debug for ListDeref<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
pub struct OffsetList<I, T: ?Sized> {
    pub offset: I,
    pub inner: T,
}

impl<I, T> ErasedList<I> for OffsetList<I, T>
where
    T: ErasedList<I>,
    I: Add<Output = I> + Copy + Send + Sync + Debug,
{
    #[inline]
    fn __erased_index(&self, index: usize) -> I {
        self.inner.__erased_index(index) + self.offset
    }

    #[inline]
    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = I> + '_)) + '_)) {
        self.inner
            .__peek_erased_iter(&mut |iter| f(&mut iter.map(|i| i + self.offset)));
    }

    #[inline]
    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = I> + 'a>
    where
        I: 'a,
    {
        Box::new(self.inner.__boxed_erased_iter().map(|i| i + self.offset))
    }

    #[inline]
    fn __erased_len(&self) -> usize {
        self.inner.__erased_len()
    }
}

impl<I, T> From<(I, T)> for OffsetList<I, T> {
    #[inline(always)]
    fn from(value: (I, T)) -> Self {
        Self {
            offset: value.0,
            inner: value.1,
        }
    }
}
