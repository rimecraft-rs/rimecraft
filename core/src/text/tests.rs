use std::sync::Arc;

use super::{
    content::{Translatable, TranslatableArg},
    Text,
};

#[test]
fn to_string() {
    let text = Text::new(
        super::content::Content::Literal(std::borrow::Cow::Borrowed("i play GENSHINE IMPACT")),
        vec![],
        Default::default(),
    );

    assert_eq!(text.to_string(), "i play GENSHINE IMPACT");
}

#[test]
fn sibs_to_string() {
    let mut text = Text::new(
        super::content::Content::Literal(std::borrow::Cow::Borrowed("i play ")),
        vec![],
        Default::default(),
    );
    text.push(Text::new(
        super::content::Content::Literal(std::borrow::Cow::Borrowed("GENSHINE")),
        vec![],
        Default::default(),
    ));
    text.push(Text::new(
        super::content::Content::Literal(std::borrow::Cow::Borrowed("")),
        vec![Text::new(
            super::content::Content::Literal(std::borrow::Cow::Borrowed(" IMPACT")),
            vec![],
            Default::default(),
        )],
        Default::default(),
    ));

    assert_eq!(text.to_string(), "i play GENSHINE IMPACT");
}

#[test]
fn ser_de() {
    let mut text = Text::new(
        super::content::Content::Literal(std::borrow::Cow::Borrowed("i play ")),
        vec![],
        Default::default(),
    );
    text.push(Text::new(
        super::content::Content::Translatable(Translatable::new(
            std::borrow::Cow::Borrowed("item.rimecraft.gold_ingot"),
            None,
            vec![TranslatableArg::Display(Arc::new(
                "玩Blue Archive的人素质都很差",
            ))],
        )),
        vec![],
        Default::default(),
    ));
    text.push(Text::new(
        super::content::Content::Literal(std::borrow::Cow::Borrowed("")),
        vec![Text::new(
            super::content::Content::Literal(std::borrow::Cow::Borrowed(" IMPACT")),
            vec![],
            Default::default(),
        )],
        Default::default(),
    ));

    let json = serde_json::to_string(&text).unwrap();
    let text2: Text = serde_json::from_str(&json).unwrap();
    assert_eq!(text.to_string(), text2.to_string())
}
