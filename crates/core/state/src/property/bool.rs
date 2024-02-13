use super::{BiIndex, Wrap};

/// Property data that wraps `true` and `false`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Data;

impl BiIndex<bool> for Data {
    #[inline]
    fn index(&self, index: isize) -> Option<bool> {
        match index {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    #[inline]
    fn index_of(&self, value: &bool) -> Option<isize> {
        Some(*value as isize)
    }
}

const VARIANTS: [bool; 2] = [false, true];

impl Wrap<bool> for Data {
    #[inline]
    fn parse_name(&self, name: &str) -> Option<bool> {
        name.parse().ok()
    }

    #[inline]
    fn to_name<'a>(&'a self, value: &bool) -> Option<std::borrow::Cow<'a, str>> {
        Some(value.to_string().into())
    }

    #[inline]
    fn variants(&self) -> usize {
        VARIANTS.len()
    }
}

impl IntoIterator for &Data {
    type Item = bool;
    type IntoIter = <[bool; 2] as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        VARIANTS.into_iter()
    }
}
