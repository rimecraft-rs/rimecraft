use std::hash::Hash;

use crate::Index;

/// A list maps ids with values.
///
/// Type `T` should be cheaply cloned (ex. [`std::sync::Arc`]).
#[derive(Clone)]
pub struct IdList<T>
where
    T: Hash + PartialEq + Eq + Clone,
{
    next_id: u32,

    id_map: std::collections::HashMap<T, u32>,
    vec: Vec<Option<T>>,
}

impl<T> IdList<T>
where
    T: Hash + PartialEq + Eq + Clone,
{
    /// Creates a new id list.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a new id list with specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            next_id: 0,
            id_map: std::collections::HashMap::with_capacity(capacity),
            vec: Vec::with_capacity(capacity),
        }
    }

    /// Replaces the target id with given value
    /// and returns the old one.
    pub fn replace(&mut self, value: T, id: u32) -> Option<T> {
        let us = id as usize;
        self.id_map.insert(value.clone(), id);

        while self.vec.len() <= us {
            self.vec.push(None);
        }

        let result = std::mem::replace(self.vec.get_mut(us).unwrap(), Some(value));

        if self.next_id <= id {
            self.next_id = id + 1;
        }

        result
    }

    /// Adds a new value into this list
    #[inline]
    pub fn add(&mut self, value: T) {
        self.replace(value, self.next_id);
    }

    /// Whether this list contains the given id.
    pub fn contains_key(&self, id: u32) -> bool {
        self.get(id as usize).is_some()
    }
}

impl<T> Default for IdList<T>
where
    T: Hash + Eq + Clone,
{
    #[inline]
    fn default() -> Self {
        Self {
            next_id: 0,
            id_map: std::collections::HashMap::new(),
            vec: vec![],
        }
    }
}

impl<T: Hash + PartialEq + Eq + Clone> Index<T> for IdList<T> {
    #[inline]
    fn index_of(&self, value: &T) -> Option<usize> {
        self.id_map.get(value).copied().map(|e| e as usize)
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&T> {
        self.vec.get(index).and_then(From::from)
    }

    #[inline]
    fn len(&self) -> usize {
        self.id_map.len()
    }
}

impl<T: Hash + PartialEq + Eq + Clone> std::ops::Index<usize> for IdList<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}
