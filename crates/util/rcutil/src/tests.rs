use crate::Any;

struct Foo;

struct Exotic<T: ?Sized>(T);

trait Abstract: Any {}

impl Abstract for Foo {}

#[test]
fn dyn_consistency() {
    let value = Foo;
    let d: &dyn Abstract = &value;
    assert_eq!(
        value.type_id_dyn(),
        d.type_id_dyn(),
        "trait object type id mismatch"
    );
}

#[test]
fn dyn_downcast() {
    let value = Foo;
    let d: &dyn Abstract = &value;
    assert!(
        unsafe { crate::try_cast_ref::<_, Foo>(d) }.is_some(),
        "failed to downcast Foo"
    );
}

#[test]
#[ignore = "needs further workarounds for exotic dst"]
fn exotic_dst_consistency() {
    let value = Exotic(Foo);
    let d: &Exotic<dyn Abstract> = &value;
    assert_eq!(
        value.type_id_dyn(),
        d.type_id_dyn(),
        "exotic dst type id mismatch"
    );
}
