use crate::Interner;

#[test]
fn intern() {
    let interner: Interner<'static, str> = Interner::new();

    interner.obtain("wow");
    interner.obtain("mom");

    assert_eq!(&*interner.obtain("wow"), "wow");
    assert_eq!(&*interner.obtain("mom"), "mom");
}
