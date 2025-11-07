use std::{borrow::Cow, ops::RangeBounds};

use super::{BiIndex, Wrap};

/// Property data that wraps a range of integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Data<T>(pub T);

impl<T> BiIndex<i32> for Data<T>
where
    T: RangeBounds<i32>,
{
    #[inline]
    fn index(&self, index: isize) -> Option<i32> {
        Some(index as i32)
    }

    #[inline]
    fn index_of(&self, value: &i32) -> Option<isize> {
        Some(*value as isize)
    }
}

impl<T> IntoIterator for &Data<T>
where
    T: ToOwned,
    <T as ToOwned>::Owned: IntoIterator,
{
    type Item = <<T as ToOwned>::Owned as IntoIterator>::Item;

    type IntoIter = <<T as ToOwned>::Owned as IntoIterator>::IntoIter;

    #[inline]
    #[allow(clippy::unnecessary_to_owned)] // wrong clippy diagnosis
    fn into_iter(self) -> Self::IntoIter {
        self.0.to_owned().into_iter()
    }
}

impl<T> Wrap<i32> for Data<T>
where
    T: RangeBounds<i32>,
{
    #[inline]
    fn parse_name(&self, name: &str) -> Option<i32> {
        name.parse().ok().filter(|val| self.0.contains(val))
    }

    #[inline]
    fn to_name<'a>(&'a self, value: &i32) -> Option<Cow<'a, str>> {
        self.0.contains(value).then_some(value.to_string().into())
    }

    fn variants(&self) -> usize {
        let end = match self.0.end_bound() {
            std::ops::Bound::Included(val) => *val,
            std::ops::Bound::Excluded(val) => *val - 1,
            std::ops::Bound::Unbounded => i32::MAX,
        };

        let start = match self.0.start_bound() {
            std::ops::Bound::Included(val) => *val,
            std::ops::Bound::Excluded(val) => *val + 1,
            std::ops::Bound::Unbounded => i32::MIN,
        };

        (end as i64 - start as i64 + 1_i64) as usize
    }
}
