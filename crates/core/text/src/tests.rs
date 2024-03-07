use crate::*;

struct Content {
    text: String,
}

impl From<&str> for Content {
    #[inline]
    fn from(value: &str) -> Self {
        Self {
            text: value.to_owned(),
        }
    }
}

impl Display for Content {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[test]
fn display() {
    let content: Content = "Hello, world! ".into();
    let mut text: Text<_, ()> = content.into();
    let mut sib: Text<_, ()> = Content::from("Genshin Impact, ").into();
    sib.push(Content::from("a game by miHoYo, ").into());
    sib.push(Content::from("boot! ").into());
    text.push(sib);
    text.push(Content::from("opssw").into());

    assert_eq!(
        text.to_string(),
        "Hello, world! Genshin Impact, a game by miHoYo, boot! opssw"
    );
}
