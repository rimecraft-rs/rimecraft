#![allow(missing_docs)]
#![cfg(test)]

use test_global::{integration::text::TextContent, TestContext};
use text::{style::Formatting, Text};

#[test]
fn serde() {
    let content: TextContent = "Hello, world! ".into();
    let mut text: Text<TestContext> = content.into();
    let mut sib: Text<TestContext> = TextContent::from("Genshin Impact, ").into();
    sib.push(TextContent::from("a game by miHoYo, ").into());
    sib.push(TextContent::from("boot! ").into());
    text.push(sib);
    text.push(TextContent::from("opssw").into());
    let style = text.style_mut();
    style.color = Some(Formatting::Aqua.try_into().unwrap());

    let nbt = fastnbt::to_bytes(&text).expect("serialization error");
    let text2: Text<TestContext> = fastnbt::from_bytes(&nbt).expect("deserialization error");

    assert_eq!(
        text2.to_string(),
        text.to_string(),
        "content should be identical"
    );
    assert_eq!(
        text2.style().color,
        text.style().color,
        "style color should be identical"
    );
}

#[test]
fn display() {
    let content: TextContent = "Hello, world! ".into();
    let mut text: Text<TestContext> = content.into();
    let mut sib: Text<TestContext> = TextContent::from("Genshin Impact, ").into();
    sib.push(TextContent::from("a game by miHoYo, ").into());
    sib.push(TextContent::from("boot! ").into());
    text.push(sib);
    text.push(TextContent::from("opssw").into());

    assert_eq!(
        text.to_string(),
        "Hello, world! Genshin Impact, a game by miHoYo, boot! opssw"
    );
}
