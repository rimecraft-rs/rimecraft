use std::hash::Hash;

/// An extended version of [`std::ops::Index`].
pub trait Indexed<T> {
    fn get_raw_id(&self, value: &T) -> Option<usize>;
    fn get(&self, index: usize) -> Option<&T>;
    fn len(&self) -> usize;
}

/// An id list, just targeting the `IdList` in MCJE.
///
/// Type `T` should be cheaply cloned (for example, an [`std::sync::Arc`]).
#[derive(Clone)]
pub struct IdList<T: Hash + PartialEq + Eq + Clone> {
    next_id: u32,
    id_map: hashbrown::HashMap<T, u32>,
    vec: Vec<Option<T>>,
}

impl<T: Hash + PartialEq + Eq + Clone> IdList<T> {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            id_map: hashbrown::HashMap::new(),
            vec: vec![],
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            next_id: 0,
            id_map: hashbrown::HashMap::with_capacity(capacity),
            vec: Vec::with_capacity(capacity),
        }
    }

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

    pub fn push(&mut self, value: T) {
        self.replace(value, self.next_id);
    }

    pub fn contains_key(&self, index: u32) -> bool {
        self.get(index as usize).is_some()
    }
}

impl<T: Hash + PartialEq + Eq + Clone> Indexed<T> for IdList<T> {
    fn get_raw_id(&self, value: &T) -> Option<usize> {
        self.id_map.get(value).copied().map(|e| e as usize)
    }

    fn get(&self, index: usize) -> Option<&T> {
        match self.vec.get(index) {
            Some(Some(e)) => Some(e),
            _ => None,
        }
    }

    fn len(&self) -> usize {
        self.id_map.len()
    }
}

impl<T: Hash + PartialEq + Eq + Clone> std::ops::Index<usize> for IdList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

/// A storage whose values are raw IDs held by palettes.
pub trait PaletteStoragge {
    fn swap(&mut self, index: u32, value: i32);
}
