use serde::{Deserialize, Serialize};

use crate::{Plain, RawText, style::Style};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[serde(bound(deserialize = r#"
    T: Deserialize<'de> + Plain,
    StyleExt: Deserialize<'de> + Default"#))]
enum Component<'a, T, StyleExt> {
    DirectLiteral(&'a str),
    List(Vec<RawText<T, StyleExt>>),
    Bool(bool),
    Integer(i64),
    Float(f64),
    Object {
        extra: Vec<RawText<T, StyleExt>>,
        #[serde(flatten)]
        style: Style<StyleExt>,
        #[serde(flatten)]
        content: T,
    },
}

impl<T, StyleExt> Serialize for RawText<T, StyleExt>
where
    T: Serialize,
    StyleExt: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Component<'a, T, StyleExt> {
            extra: &'a [RawText<T, StyleExt>],
            #[serde(flatten)]
            style: &'a Style<StyleExt>,
            #[serde(flatten)]
            content: &'a T,
        }

        Component {
            content: &self.content,
            style: &self.style,
            extra: &self.sibs,
        }
        .serialize(serializer)
    }
}

impl<'de, T, StyleExt> Deserialize<'de> for RawText<T, StyleExt>
where
    T: Deserialize<'de> + Plain,
    StyleExt: Deserialize<'de> + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match <Component<'de, T, StyleExt>>::deserialize(deserializer)? {
            Component::DirectLiteral(val) => Ok(T::from_literal(val).into()),
            Component::List(mut ls) => {
                if ls.is_empty() {
                    return Err(serde::de::Error::custom("empty list"));
                }
                let mut text = ls.remove(0);
                text.sibs = ls;
                Ok(text)
            }
            Component::Bool(val) => Ok(T::from_literal(if val { "true" } else { "false" }).into()),
            Component::Integer(val) => Ok(T::from_literal(&val.to_string()).into()),
            Component::Float(val) => Ok(T::from_literal(&val.to_string()).into()),
            Component::Object {
                content,
                style,
                extra,
            } => Ok(Self {
                content,
                style,
                sibs: extra,
            }),
        }
    }
}
