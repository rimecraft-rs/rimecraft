use std::{fmt::Debug, iter::FusedIterator};

use global_cx::GlobalContext;
use local_cx::ProvideLocalCxTy;

use crate::data::{ErasedEntry, SerializedEntry};

/// Iterator over all changed entries of a `DataTracker`.
pub struct ChangedEntries<'borrow, 'a, Cx>
where
    Cx: ProvideLocalCxTy,
{
    pub(crate) inner_iter: std::slice::Iter<'borrow, ErasedEntry<'a, Cx>>,
}

impl<Cx> FusedIterator for ChangedEntries<'_, '_, Cx> where Cx: ProvideLocalCxTy + GlobalContext {}

impl<'borrow, 'a, Cx> Iterator for ChangedEntries<'borrow, 'a, Cx>
where
    Cx: ProvideLocalCxTy + GlobalContext,
{
    type Item = SerializedEntry<'borrow, 'a, Cx>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.inner_iter.next()?;
            if !next.is_unchanged() {
                return Some(next.as_serialized());
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.inner_iter.len()))
    }
}

impl<Cx> Debug for ChangedEntries<'_, '_, Cx>
where
    Cx: ProvideLocalCxTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangedEntries")
            .field("len", &self.inner_iter.len())
            .finish_non_exhaustive()
    }
}

/// Iterator over all dirty entries of a `DataTracker`.
///
/// See [`SelfCleaningDirtyEntries`] if you want to automatically clear the dirty flag like the way
/// it is done in vanilla.
pub struct DirtyEntries<'borrow, 'a, Cx>
where
    Cx: ProvideLocalCxTy,
{
    pub(crate) inner_iter: std::slice::Iter<'borrow, ErasedEntry<'a, Cx>>,
}

impl<Cx> FusedIterator for DirtyEntries<'_, '_, Cx> where Cx: ProvideLocalCxTy + GlobalContext {}

impl<'borrow, 'a, Cx> Iterator for DirtyEntries<'borrow, 'a, Cx>
where
    Cx: ProvideLocalCxTy + GlobalContext,
{
    type Item = SerializedEntry<'borrow, 'a, Cx>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.inner_iter.next()?;
            if next.is_dirty() {
                return Some(next.as_serialized());
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.inner_iter.len()))
    }
}

impl<Cx> Debug for DirtyEntries<'_, '_, Cx>
where
    Cx: ProvideLocalCxTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirtyEntries")
            .field("len", &self.inner_iter.len())
            .finish_non_exhaustive()
    }
}

/// Iterator over all dirty entries of a `DataTracker` that _automatically clears the dirty flag._
pub struct SelfCleaningDirtyEntries<'borrow, 'a, Cx>
where
    Cx: ProvideLocalCxTy,
{
    pub(crate) inner_iter: std::slice::IterMut<'borrow, ErasedEntry<'a, Cx>>,
}

impl<Cx> FusedIterator for SelfCleaningDirtyEntries<'_, '_, Cx> where
    Cx: ProvideLocalCxTy + GlobalContext
{
}

impl<'borrow, 'a, Cx> Iterator for SelfCleaningDirtyEntries<'borrow, 'a, Cx>
where
    Cx: ProvideLocalCxTy + GlobalContext,
{
    type Item = SerializedEntry<'borrow, 'a, Cx>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.inner_iter.next()?;
            if next.is_dirty() {
                next.set_dirty(false);
                return Some(next.as_serialized());
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.inner_iter.len()))
    }
}

impl<Cx> Debug for SelfCleaningDirtyEntries<'_, '_, Cx>
where
    Cx: ProvideLocalCxTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelfCleaningDirtyEntries")
            .field("len", &self.inner_iter.len())
            .finish_non_exhaustive()
    }
}
