use std::{any::TypeId, collections::HashSet};

use crate::{DescriptorSet, Type};

type Id = &'static str;

#[derive(Default)]
struct DummyRegistry {
    set: HashSet<(Id, TypeId)>,
    ptr: Box<usize>,
}

unsafe impl super::Registry for DummyRegistry {
    type Identifier = Id;

    fn register<T>(&mut self, id: Self::Identifier) -> Option<usize> {
        self.set
            .insert((id, typeid::of::<T>()))
            .then_some(self.set.len() - 1)
    }

    fn marker(&self) -> *const () {
        std::ptr::from_ref(self.ptr.as_ref()) as *const ()
    }
}

#[test]
fn registry() {
    let mut registry = DummyRegistry::default();
    let _ty = Type::<fn()>::new(&mut registry, "foo").expect("register failed");
}

#[test]
fn builder() {
    let mut registry = DummyRegistry::default();
    let ty = Type::<fn() -> u8>::new(&mut registry, "foo").expect("register failed");
    let mut builder = DescriptorSet::builder();
    builder.insert(ty, || 114u8);
    let set = builder.build();
    assert_eq!(set.get(ty).expect("get failed")(), 114u8, "wrong function")
}
