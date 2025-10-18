//! Integration tests for the Localize derive macro.

#![allow(unused)]

use rimecraft_text::Localize;

#[derive(Localize)]
enum SimpleEnum {
    First,
    Second,
    Third,
}

#[test]
fn test_simple_enum_default() {
    assert_eq!(SimpleEnum::First.localization_key(), "first");
    assert_eq!(SimpleEnum::Second.localization_key(), "second");
    assert_eq!(SimpleEnum::Third.localization_key(), "third");
}

#[derive(Localize)]
enum WithExplicitSegments {
    #[localize(options, particles, all)]
    All,

    #[localize(options, particles, decreased)]
    Decreased,

    #[localize(options, particles, minimal)]
    Minimal,
}

#[test]
fn test_explicit_segments() {
    assert_eq!(
        WithExplicitSegments::All.localization_key(),
        "options.particles.all"
    );
    assert_eq!(
        WithExplicitSegments::Decreased.localization_key(),
        "options.particles.decreased"
    );
    assert_eq!(
        WithExplicitSegments::Minimal.localization_key(),
        "options.particles.minimal"
    );
}

#[derive(Localize)]
enum WithUnderscorePlaceholder {
    #[localize(options, difficulty, _)]
    Peaceful,

    #[localize(options, difficulty, _)]
    Easy,

    #[localize(options, difficulty, _)]
    Normal,

    #[localize(options, difficulty, _)]
    Hard,
}

#[test]
fn test_underscore_placeholder() {
    assert_eq!(
        WithUnderscorePlaceholder::Peaceful.localization_key(),
        "options.difficulty.peaceful"
    );
    assert_eq!(
        WithUnderscorePlaceholder::Easy.localization_key(),
        "options.difficulty.easy"
    );
    assert_eq!(
        WithUnderscorePlaceholder::Normal.localization_key(),
        "options.difficulty.normal"
    );
    assert_eq!(
        WithUnderscorePlaceholder::Hard.localization_key(),
        "options.difficulty.hard"
    );
}

#[derive(Localize)]
enum MixedFormats {
    #[localize(category, _, suffix)]
    First,

    #[localize = "custom.static.key"]
    Second,

    #[localize]
    Third,

    Default,
}

#[test]
fn test_mixed_formats() {
    assert_eq!(
        MixedFormats::First.localization_key(),
        "category.first.suffix"
    );
    assert_eq!(MixedFormats::Second.localization_key(), "custom.static.key");
    assert_eq!(MixedFormats::Third.localization_key(), "third");
    assert_eq!(MixedFormats::Default.localization_key(), "default");
}

#[derive(Localize)]
enum WithStringLiterals {
    #[localize = "status.online"]
    Online,

    #[localize = "status._.busy"]
    Away,

    #[localize = "status.offline._.detail"]
    Offline,
}

#[test]
fn test_string_literals() {
    assert_eq!(
        WithStringLiterals::Online.localization_key(),
        "status.online"
    );
    assert_eq!(
        WithStringLiterals::Away.localization_key(),
        "status.away.busy"
    );
    assert_eq!(
        WithStringLiterals::Offline.localization_key(),
        "status.offline.offline.detail"
    );
}

#[derive(Localize)]
enum CamelCaseVariants {
    #[localize(test, _)]
    HTTPServer,

    #[localize(test, _)]
    IOError,

    #[localize(test, _)]
    SimpleCase,
}

#[test]
fn test_camel_case_conversion() {
    assert_eq!(
        CamelCaseVariants::HTTPServer.localization_key(),
        "test.http_server"
    );
    assert_eq!(
        CamelCaseVariants::IOError.localization_key(),
        "test.io_error"
    );
    assert_eq!(
        CamelCaseVariants::SimpleCase.localization_key(),
        "test.simple_case"
    );
}

#[derive(Localize)]
enum EnumWithData {
    #[localize(variant, unit)]
    Unit,

    #[localize(variant, tuple)]
    Tuple(i32, String),

    #[localize(variant, named)]
    Named { x: i32, y: String },
}

#[test]
fn test_enum_with_data() {
    assert_eq!(EnumWithData::Unit.localization_key(), "variant.unit");
    assert_eq!(
        EnumWithData::Tuple(42, "test".to_owned()).localization_key(),
        "variant.tuple"
    );
    assert_eq!(
        EnumWithData::Named {
            x: 42,
            y: "test".to_owned()
        }
        .localization_key(),
        "variant.named"
    );
}

#[derive(Localize)]
enum OnlyUnderscores {
    #[localize(_)]
    First,

    #[localize(_)]
    Second,
}

#[test]
fn test_only_underscores() {
    assert_eq!(OnlyUnderscores::First.localization_key(), "first");
    assert_eq!(OnlyUnderscores::Second.localization_key(), "second");
}

#[derive(Localize)]
enum ComplexSegments {
    #[localize(prefix, middle, _, suffix, end)]
    ComplexVariant,
}

#[test]
fn test_complex_segments() {
    assert_eq!(
        ComplexSegments::ComplexVariant.localization_key(),
        "prefix.middle.complex_variant.suffix.end"
    );
}
