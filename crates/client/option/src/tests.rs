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
    let mut bool_option = bool::<TestContext>(
        TextContent::from("test_bool_option").into(),
        Box::new(|_: &Text<TestContext>, b: &bool| TextContent::from(b.to_string()).into()),
        true,
        Box::new(DynamicTooltipFactory::new(|b: &bool| {
            Some(Tooltip::from(Text::<TestContext>::from(TextContent::from(
                b.to_string(),
            ))))
        })),
        Box::new(|b: &bool| println!("Bool option changed to: {}", b)),
    );

    assert_eq!(
        bool_option
            .value_text_getter
            .value_text(&bool_option.text, &true)
            .to_string(),
        "true"
    );

    assert_eq!(
        bool_option
            .tooltip_factory
            .apply(&true)
            .map(|t| t.into_items()),
        Some(Tooltip::from(Text::<TestContext>::from(TextContent::from("true"))).into_items())
    );

    // Change the value and check the callback

    println!("Setting bool_option to false:");
    bool_option.set_value(false);
    println!("(should have printed the change callback above)");
    assert!(!bool_option.value);

    println!();

    println!("Setting bool_option to false:");
    bool_option.set_value(false);
    println!("(should NOT have printed the change callback above)");
    assert!(!bool_option.value);
}
