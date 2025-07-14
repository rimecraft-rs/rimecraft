use crate::{DescriptorSet, Type, simple_registry::SimpleRegistry};

type Id = &'static str;

#[test]
fn registry() {
    let mut registry = SimpleRegistry::<Id>::default();
    let _ty = Type::<fn()>::new(&mut registry, "foo").expect("register failed");
}

#[test]
fn builder() {
    let mut registry = SimpleRegistry::<Id>::default();
    let ty = Type::<fn() -> u8>::new(&mut registry, "foo").expect("register failed");
    let mut builder = DescriptorSet::builder();
    builder.insert(ty, || 114u8);
    let set = builder.build();
    assert_eq!(set.get(ty).expect("get failed")(), 114u8, "wrong function")
}
