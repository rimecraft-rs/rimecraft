pub trait IndexedIterable<T>: Sized {
    fn get_raw_id<'a>(&'a self, object: &'a T) -> Option<usize>;
    fn get_from_raw_id(&self, id: usize) -> Option<&T>;
    fn get_from_raw_id_mut(&mut self, id: usize) -> Option<&mut T>;
    fn size(&self) -> usize;
    fn vec(&self) -> Vec<&T>;
    fn vec_mut(&mut self) -> Vec<&mut T>;
}
