#![cfg(test)]

use crate::{BaseLocalContext, LocalContext};

struct DummyCx {
    msg: u32,
    info: String,
}

impl BaseLocalContext for &DummyCx {}

impl LocalContext<u32> for &DummyCx {
    fn acquire(self) -> u32 {
        self.msg
    }
}

impl<'a> LocalContext<&'a str> for &'a DummyCx {
    fn acquire(self) -> &'a str {
        &self.info
    }
}

#[test]
#[cfg(feature = "dyn-cx")]
fn dyn_context() {
    use crate::dyn_cx::{ContextTable, DynamicContext};

    let context_raw = DummyCx {
        msg: 114,
        info: "hello".to_owned(),
    };

    let mut table = ContextTable::new();
    table.enable::<u32>();
    table.enable::<&str>();
    let table: ContextTable<&DummyCx> = table;

    let dyn_cx = DynamicContext::new(&context_raw, table);
    let erased = unsafe { dyn_cx.as_unsafe_cx() };

    assert_eq!(
        LocalContext::<u32>::acquire(&context_raw),
        LocalContext::<u32>::acquire(erased),
        "msg value mismatch"
    );

    assert_eq!(
        LocalContext::<&str>::acquire(&context_raw),
        LocalContext::<&str>::acquire(erased),
        "info value mismatch"
    );
}
