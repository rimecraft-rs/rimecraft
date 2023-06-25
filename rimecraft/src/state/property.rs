#[derive(Eq, Clone, Debug)]
pub struct Property {
    name: String,
    range: (u8, u8),
    type_id: std::any::TypeId,
}

impl Property {
    pub fn new<T: Into<u8> + 'static>(name: String, range: (T, T)) -> Self {
        Self {
            name,
            range: (range.0.into(), range.1.into()),
            type_id: std::any::TypeId::of::<T>(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn range(&self) -> (u8, u8) {
        self.range
    }

    pub fn values<T: From<u8>>(&self) -> Vec<T> {
        let mut vec = Vec::new();
        for i in self.range.0..=self.range.1 {
            vec.push(i.into())
        }
        vec
    }
}

impl std::hash::Hash for Property {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.type_id.hash(state);
    }
}

impl PartialEq for Property {
    fn eq(&self, other: &Self) -> bool {
        // impl [`Eq`] for property just for hash mapping. based
        other.name == self.name && self.type_id == other.type_id
    }
}
