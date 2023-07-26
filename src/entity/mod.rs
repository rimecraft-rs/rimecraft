mod event;

pub use event::*;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Type {
    id: usize,
}

impl crate::registry::Registration for Type {
    fn accept(&mut self, id: usize) {
        self.id = id
    }

    fn raw_id(&self) -> usize {
        self.id
    }
}

pub struct Entity {
    entity_type: Type,
    uuid: uuid::Uuid,
}
