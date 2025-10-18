use rimecraft_test_global::{TestContext, integration::text::TextContent};

use crate::*;

#[test]
fn test_callbacks() {
    let bool_callbacks = callbacks::bool::<TestContext>();

    assert_eq!(bool_callbacks.validate(true), Some(true));
    assert_eq!(bool_callbacks.validate(false), Some(false));
}

#[test]
fn test_simple_options() {
    let bool_option = bool::<TestContext>(
        TextContent::from("test_bool_option").into(),
        Box::new(|_, b: &bool| TextContent::from(&b.to_string()[..]).into()),
        true,
        Box::new(|_| None),
        Box::new(|b: &bool| println!("Bool option changed to: {}", b)),
    );
}
