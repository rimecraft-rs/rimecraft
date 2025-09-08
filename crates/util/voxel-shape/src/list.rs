use std::{
    any::TypeId,
    fmt::Debug,
    ops::{Add, Deref},
};

use crate::F64_TOLERANCE;

pub trait List<T> {
    fn index(&self, index: usize) -> T;
    fn iter(&self) -> impl IntoIterator<Item = T>;
    fn len(&self) -> usize;

    #[inline]
    fn downcast_fractional_double_list(&self) -> Option<&FractionalDoubleList> {
        None
    }
}

pub trait ErasedList<T>: Send + Sync + Debug {
    fn __erased_index(&self, index: usize) -> T;

    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = T> + '_)) + '_));

    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a>
    where
        T: 'a;

    fn __erased_len(&self) -> usize;

    #[inline]
    fn __downcast_fractional_double_list(&self) -> Option<&FractionalDoubleList> {
        None
    }

    fn __is_identical(&self, other: &dyn ErasedList<T>) -> bool {
        let _ = other;
        false
    }

    fn typeid(&self) -> TypeId {
        typeid::of::<Self>()
    }
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

    #[inline]
    fn downcast_fractional_double_list(&self) -> Option<&FractionalDoubleList> {
        (**self).downcast_fractional_double_list()
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

    #[inline]
    fn len(&self) -> usize {
        self.section_count + 1
    }

    #[inline]
    fn downcast_fractional_double_list(&self) -> Option<&FractionalDoubleList> {
        Some(self)
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

    #[inline]
    fn __downcast_fractional_double_list(&self) -> Option<&FractionalDoubleList> {
        self.0.__downcast_fractional_double_list()
    }

    fn __is_identical(&self, other: &dyn ErasedList<I>) -> bool {
        self.typeid() == other.typeid()
            && std::ptr::eq(
                &*self.0,
                //SAFETY: type checked and we don't care about lifetime.
                // soundness is another issue but literally causes nothing bad
                &*unsafe { &*std::ptr::from_ref(other).cast::<Self>() }.0,
            )
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

    #[inline]
    fn __downcast_fractional_double_list(&self) -> Option<&FractionalDoubleList> {
        self.0.downcast_fractional_double_list()
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

#[derive(Debug, Clone, Copy)]
pub struct PairListIterItem {
    pub x: u32,
    pub y: u32,
    pub index: usize,
}

pub trait PairErasedList<T>: ErasedList<T> {
    fn __peek_pair_erased_iter(
        &self,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = PairListIterItem> + '_)) + '_),
    );

    #[inline]
    fn __downcast_fractional_pair_double_list(&self) -> Option<&FractionalPairDoubleList> {
        None
    }
}

#[derive(Debug)]
pub struct IdentityPairList<T>(pub T);

impl<I, T> ErasedList<I> for IdentityPairList<T>
where
    T: ErasedList<I>,
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

    #[inline]
    fn __downcast_fractional_double_list(&self) -> Option<&FractionalDoubleList> {
        self.0.__downcast_fractional_double_list()
    }
}

impl<I, T> PairErasedList<I> for IdentityPairList<T>
where
    T: ErasedList<I>,
{
    fn __peek_pair_erased_iter(
        &self,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = PairListIterItem> + '_)) + '_),
    ) {
        f(
            &mut (0..self.0.__erased_len() - 1).map(|i| PairListIterItem {
                x: i as u32,
                y: i as u32,
                index: i,
            }),
        )
    }
}

#[derive(Debug)]
pub struct SimplePairDoubleList {
    indices: Box<[f64]>,
    min_vals: Box<[u32]>,
    max_vals: Box<[u32]>,
    len: usize,
}

impl SimplePairDoubleList {
    pub fn new(
        lhs: &(dyn ErasedList<f64> + '_),
        rhs: &(dyn ErasedList<f64> + '_),
        lhs_only: bool, // include lhs-only elements
        rhs_only: bool, // include rhs-only elements
    ) -> Self {
        let mut d = f64::NEG_INFINITY;
        let len_lhs = lhs.__erased_len();
        let len_rhs = rhs.__erased_len();
        let len_merged = len_lhs + len_rhs;
        let mut this = Self {
            indices: vec![0f64; len_merged].into_boxed_slice(),
            min_vals: vec![0u32; len_merged].into_boxed_slice(),
            max_vals: vec![0u32; len_merged].into_boxed_slice(),
            len: 0,
        };
        let mut indices_ptr = 0usize;
        let mut lhs_ptr = 0usize;
        let mut rhs_ptr = 0usize;

        loop {
            let reached_lhs = lhs_ptr >= len_lhs;
            let reached_rhs = rhs_ptr >= len_rhs;
            if reached_lhs && reached_rhs {
                this.len = indices_ptr.max(1);
                break;
            }

            // Smaller element first
            let make_lhs = !reached_lhs
                && (reached_rhs
                    || lhs.__erased_index(lhs_ptr) < rhs.__erased_index(rhs_ptr) + F64_TOLERANCE);

            if make_lhs {
                lhs_ptr += 1;
                // skip if rhs is over and lhs-only isn't tolerant
                if !lhs_only && (rhs_ptr == 0 || reached_rhs) {
                    continue;
                }
            } else {
                rhs_ptr += 1;
                // skip if lhs is over and rhs-only isn't tolerant
                if !rhs_only && (lhs_ptr == 0 || reached_lhs) {
                    continue;
                }
            }

            let lhs_ptr_current = lhs_ptr.overflowing_sub(1).0;
            let rhs_ptr_current = rhs_ptr.overflowing_sub(1).0;
            let e = if make_lhs {
                lhs.__erased_index(lhs_ptr_current)
            } else {
                rhs.__erased_index(rhs_ptr_current)
            };
            if d < e - F64_TOLERANCE {
                this.min_vals[indices_ptr] = lhs_ptr_current as u32;
                this.max_vals[indices_ptr] = rhs_ptr_current as u32;
                this.indices[indices_ptr] = e;
                indices_ptr += 1;
                d = e;
            } else {
                this.min_vals[indices_ptr - 1] = lhs_ptr_current as u32;
                this.max_vals[indices_ptr - 1] = rhs_ptr_current as u32;
            }
        }

        this
    }
}

impl ErasedList<f64> for SimplePairDoubleList {
    #[inline]
    fn __erased_index(&self, index: usize) -> f64 {
        ListEraser(&self.indices[..self.len]).__erased_index(index)
    }

    #[inline]
    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = f64> + '_)) + '_)) {
        ListEraser(&self.indices[..self.len]).__peek_erased_iter(f)
    }

    #[inline]
    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = f64> + 'a>
    where
        f64: 'a,
    {
        Box::new(self.indices[..self.len].iter().copied())
    }

    #[inline]
    fn __erased_len(&self) -> usize {
        self.len
    }
}

impl PairErasedList<f64> for SimplePairDoubleList {
    fn __peek_pair_erased_iter(
        &self,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = PairListIterItem> + '_)) + '_),
    ) {
        f(&mut self.min_vals[..self.len - 1]
            .iter()
            .copied()
            .zip(self.max_vals[..self.len - 1].iter().copied())
            .enumerate()
            .map(|(index, (x, y))| PairListIterItem { x, y, index }))
    }
}

#[derive(Debug)]
pub struct FractionalPairDoubleList {
    list: FractionalDoubleList,
    first_sec_count: u32,
    gcd: u32,
}

impl FractionalPairDoubleList {
    pub fn new(i: usize, j: usize) -> Self {
        let (gcd, lcm) = math::int::gcd_lcm(i, j);
        Self {
            list: FractionalDoubleList { section_count: lcm },
            first_sec_count: (i / gcd) as u32,
            gcd: (j / gcd) as u32,
        }
    }
}

impl ErasedList<f64> for FractionalPairDoubleList {
    #[inline]
    fn __erased_index(&self, index: usize) -> f64 {
        self.list.index(index)
    }

    #[inline]
    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = f64> + '_)) + '_)) {
        f(&mut self.list.iter().into_iter())
    }

    #[inline]
    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = f64> + 'a>
    where
        f64: 'a,
    {
        Box::new(self.list.iter().into_iter())
    }

    #[inline]
    fn __erased_len(&self) -> usize {
        self.list.len()
    }
}

impl PairErasedList<f64> for FractionalPairDoubleList {
    fn __peek_pair_erased_iter(
        &self,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = PairListIterItem> + '_)) + '_),
    ) {
        f(&mut (0..self.list.len() - 1).map(|i| PairListIterItem {
            x: i as u32 / self.gcd,
            y: i as u32 / self.first_sec_count,
            index: i,
        }))
    }

    #[inline]
    fn __downcast_fractional_pair_double_list(&self) -> Option<&FractionalPairDoubleList> {
        Some(self)
    }
}

#[derive(Debug)]
#[doc(alias = "DisjointPairList")]
pub struct ChainedPairList<L, R = L> {
    pub left: R,
    pub right: L,
    pub inverted: bool,
}

impl<L, R, I> ErasedList<I> for ChainedPairList<L, R>
where
    L: ErasedList<I>,
    R: ErasedList<I>,
{
    fn __erased_index(&self, index: usize) -> I {
        if index < self.left.__erased_len() {
            self.left.__erased_index(index)
        } else {
            self.right.__erased_index(index - self.left.__erased_len())
        }
    }

    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = I> + '_)) + '_)) {
        self.left.__peek_erased_iter(&mut |li| {
            self.right
                .__peek_erased_iter(&mut |ri| f(&mut li.chain(ri)))
        });
    }

    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = I> + 'a>
    where
        I: 'a,
    {
        Box::new(
            self.left
                .__boxed_erased_iter()
                .chain(self.right.__boxed_erased_iter()),
        )
    }

    #[inline]
    fn __erased_len(&self) -> usize {
        self.left.__erased_len() + self.right.__erased_len()
    }
}

impl<L, R, I> PairErasedList<I> for ChainedPairList<L, R>
where
    L: ErasedList<I>,
    R: ErasedList<I>,
{
    fn __peek_pair_erased_iter(
        &self,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = PairListIterItem> + '_)) + '_),
    ) {
        let len_l = self.left.__erased_len();
        let len_r = self.right.__erased_len();

        let base = (0..len_l).map(|j| (j as u32, u32::MAX, j)).chain(
            (0..len_r - 1).map(|k| (len_l.overflowing_sub(1).0 as u32, k as u32, len_r + k)),
        );

        if self.inverted {
            f(&mut base.map(|(x, y, index)| PairListIterItem { x: y, y: x, index }))
        } else {
            f(&mut base.map(|(x, y, index)| PairListIterItem { x, y, index }))
        }
    }
}

#[derive(Debug)]
pub enum FastDoublePairList<T> {
    Identity(IdentityPairList<T>),
    Simple(SimplePairDoubleList),
    Fractional(FractionalPairDoubleList),
    Chained(ChainedPairList<T, T>),
}

impl<T> FastDoublePairList<T>
where
    T: ErasedList<f64> + 'static,
{
    pub fn into_boxed_list(self) -> Box<dyn ErasedList<f64>> {
        match self {
            Self::Identity(i) => Box::new(i),
            Self::Simple(i) => Box::new(i),
            Self::Fractional(i) => Box::new(i),
            Self::Chained(i) => Box::new(i),
        }
    }
}

impl<T> ErasedList<f64> for FastDoublePairList<T>
where
    T: ErasedList<f64>,
{
    fn __erased_index(&self, index: usize) -> f64 {
        match self {
            Self::Identity(i) => i.__erased_index(index),
            Self::Simple(i) => i.__erased_index(index),
            Self::Fractional(i) => i.__erased_index(index),
            Self::Chained(i) => i.__erased_index(index),
        }
    }

    fn __peek_erased_iter(&self, f: &mut (dyn FnMut(&mut (dyn Iterator<Item = f64> + '_)) + '_)) {
        match self {
            Self::Identity(i) => i.__peek_erased_iter(f),
            Self::Simple(i) => i.__peek_erased_iter(f),
            Self::Fractional(i) => i.__peek_erased_iter(f),
            Self::Chained(i) => i.__peek_erased_iter(f),
        }
    }

    fn __boxed_erased_iter<'a>(&'a self) -> Box<dyn Iterator<Item = f64> + 'a>
    where
        f64: 'a,
    {
        match self {
            Self::Identity(i) => i.__boxed_erased_iter(),
            Self::Simple(i) => i.__boxed_erased_iter(),
            Self::Fractional(i) => i.__boxed_erased_iter(),
            Self::Chained(i) => i.__boxed_erased_iter(),
        }
    }

    fn __erased_len(&self) -> usize {
        match self {
            Self::Identity(i) => i.__erased_len(),
            Self::Simple(i) => i.__erased_len(),
            Self::Fractional(i) => i.__erased_len(),
            Self::Chained(i) => i.__erased_len(),
        }
    }
}

impl<T> PairErasedList<f64> for FastDoublePairList<T>
where
    T: ErasedList<f64>,
{
    fn __peek_pair_erased_iter(
        &self,
        f: &mut (dyn FnMut(&mut (dyn Iterator<Item = PairListIterItem> + '_)) + '_),
    ) {
        match self {
            Self::Identity(i) => i.__peek_pair_erased_iter(f),
            Self::Simple(i) => i.__peek_pair_erased_iter(f),
            Self::Fractional(i) => i.__peek_pair_erased_iter(f),
            Self::Chained(i) => i.__peek_pair_erased_iter(f),
        }
    }

    fn __downcast_fractional_pair_double_list(&self) -> Option<&FractionalPairDoubleList> {
        match self {
            Self::Fractional(i) => Some(i),
            _ => None,
        }
    }
}
