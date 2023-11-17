use super::Text;

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
        crate::text::content::Content::Empty,
        vec![Text::new(
            super::content::Content::Literal(std::borrow::Cow::Borrowed(
                "玩Blue Archive的人素质都很差",
                // 狠狠同意了
            )),
            vec![],
            Default::default(),
        )],
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
