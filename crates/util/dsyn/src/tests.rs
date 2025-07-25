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

#[test]
fn parenting() {
    let mut registry = SimpleRegistry::<Id>::default();
    let ty = Type::<fn() -> u8>::new(&mut registry, "foo").expect("register failed");
    let mut builder = DescriptorSet::builder();
    builder.insert(ty, || 114u8);
    let set = builder.build();
    let builder = DescriptorSet::builder_with_parent(&set);
    let set2 = builder.build();
    assert_eq!(
        set.get(ty).expect("get failed")(),
        set2.get(ty).expect("get failed")(),
        "function mismatch"
    )
}

#[test]
fn parenting_overrides() {
    let mut registry = SimpleRegistry::<Id>::default();
    let ty = Type::<fn() -> u8>::new(&mut registry, "foo").expect("register failed");
    let mut builder = DescriptorSet::builder();
    builder.insert(ty, || 114u8);
    let set = builder.build();
    let mut builder = DescriptorSet::builder_with_parent(&set);
    builder.insert(ty, || 14u8);
    let set2 = builder.build();
    assert_ne!(
        set.get(ty).expect("get failed")(),
        set2.get(ty).expect("get failed")(),
        "function matches"
    );
    assert_eq!(set2.get(ty).expect("get failed")(), 14u8, "wrong function")
}

#[test]
fn parenting_flat() {
    let mut registry = SimpleRegistry::<Id>::default();
    let ty = Type::<fn() -> u8>::new(&mut registry, "foo").expect("register failed");
    let mut builder = DescriptorSet::builder();
    builder.insert(ty, || 114u8);
    let set = builder.build();
    let builder = DescriptorSet::builder_with_parent(&set);
    let set2 = builder.flatten().build();
    assert_eq!(
        set.get(ty).expect("get failed")(),
        set2.get(ty).expect("get failed")(),
        "function mismatch"
    )
}

#[test]
fn parenting_overrides_flat() {
    let mut registry = SimpleRegistry::<Id>::default();
    let ty = Type::<fn() -> u8>::new(&mut registry, "foo").expect("register failed");
    let mut builder = DescriptorSet::builder();
    builder.insert(ty, || 114u8);
    let set = builder.build();
    let mut builder = DescriptorSet::builder_with_parent(&set);
    builder.insert(ty, || 14u8);
    let set2 = builder.flatten().build();
    assert_ne!(
        set.get(ty).expect("get failed")(),
        set2.get(ty).expect("get failed")(),
        "function matches"
    );
    assert_eq!(set2.get(ty).expect("get failed")(), 14u8, "wrong function")
}
