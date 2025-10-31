//! Random example.

use rimecraft_remap::{remap, remap_fn};

fn main() {
    assert_eq!(id::<(), ()>("a"), "a");
    assert_eq!(resource_location::<(), ()>("a"), "a");
    assert_eq!(identifier::<(), ()>("d"), "d");
}

#[remap(mojmaps = "Receiver")]
trait Recv<'a> {
    type Foo;
}

impl Recv<'_> for () {
    type Foo = fn() -> i32;
}

/// Docs
#[remap_fn(mojmaps = "resourceLocation", yarn = "identifier")]
fn id<'a, T: Recv<'a>, B: Recv<'a, Foo = fn() -> i32>>(s: &'a str) -> &'a str {
    s
}

#[remap(mojmaps = "ResourceLocation", yarn = "Identifier")]
pub(crate) struct Id;
