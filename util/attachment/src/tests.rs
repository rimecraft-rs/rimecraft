use crate::{Attachments, Simple, Type};

#[test]
fn attach_get() {
    const INT_TYPE: Type<&'static str, Simple<i32>> = Type::new("dummy_int");
    const STRING_TYPE: Type<&'static str, Simple<String>> = Type::new("dummy_string");
    let mut attachments: Attachments<String> = Attachments::new();

    // Attach data
    attachments.attach(&INT_TYPE, Simple(16)).unwrap();
    attachments
        .attach(&STRING_TYPE, Simple("WoW".to_owned()))
        .unwrap();

    // Obtain data
    assert_eq!(
        attachments.get::<Simple<i32>, _>(&INT_TYPE).copied(),
        Some(16)
    );
    assert_eq!(
        attachments
            .get::<Simple<String>, _>(&STRING_TYPE)
            .map(String::as_str),
        Some("WoW")
    );
}

#[test]
#[cfg(feature = "serde")]
fn serde() {
    use std::time::Duration;

    use rimecraft_serde_update::Update;

    use crate::serde::Persistent;

    const DURATION_TYPE: Type<&'static str, Persistent<Duration>> = Type::new("duration");
    const STRING_VEC_TYPE: Type<&'static str, Persistent<Vec<String>>> = Type::new("string");
    const INT_TYPE: Type<&'static str, Simple<i32>> = Type::new("dummy_int");
    let mut attachments: Attachments<String> = Attachments::new();

    // Attach data
    attachments
        .attach(&DURATION_TYPE, Persistent::new(Duration::from_secs(16)))
        .unwrap();
    attachments
        .attach(
            &STRING_VEC_TYPE,
            Persistent::new(vec!["WoW".to_owned(), "Rust".to_owned()]),
        )
        .unwrap();
    attachments.attach(&INT_TYPE, Simple(64)).unwrap();

    // Serialize
    let serialized = fastnbt::to_value(&attachments).unwrap();

    // Fake clone
    let mut attachments: Attachments<String> = Attachments::new();
    attachments
        .attach(&DURATION_TYPE, Persistent::new(Duration::from_secs(114514)))
        .unwrap();
    attachments
        .attach(
            &STRING_VEC_TYPE,
            Persistent::new(vec!["Oreo".to_owned(), "Java".to_owned()]),
        )
        .unwrap();
    attachments.attach(&INT_TYPE, Simple(256)).unwrap();

    // Update
    attachments.update(&serialized).unwrap();

    // Obtain data
    assert_eq!(
        attachments
            .get::<Persistent<Duration>, _>(&DURATION_TYPE)
            .map(|x| x.as_secs()),
        Some(16)
    );
    assert_eq!(
        attachments
            .get::<Persistent<Vec<String>>, _>(&STRING_VEC_TYPE)
            .map(|x| x.to_vec()),
        Some(vec!["WoW".to_owned(), "Rust".to_owned()])
    );
    assert_eq!(
        attachments.get::<Simple<i32>, _>(&INT_TYPE).copied(),
        Some(256)
    );
}
