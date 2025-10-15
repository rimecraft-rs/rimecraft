use rimecraft_fmt::Formatting;
use style::Color;

use crate::*;

#[test]
fn color_serde() {
    let color = Color::try_from(Formatting::Aqua).expect("expected to parse aqua color");
    assert_eq!(color.name().to_string(), "aqua", "name should be 'aqua'");
    assert_eq!(
        color
            .name()
            .parse::<Color>()
            .expect("aqua should be parsed from string correctly"),
        color,
        "aqua should be parsed from string correctly"
    );

    let color: Color = color.rgb().into();
    assert_ne!(
        color.name().to_string(),
        "aqua",
        "name should not be 'aqua'"
    );
    assert_eq!(
        color
            .name()
            .parse::<Color>()
            .expect("rgb should be parsed from string correctly"),
        color,
        "rgb should be parsed from string correctly"
    );
}
