use serde::{Deserialize, Serialize};

use crate::text::content::{Content, Translatable, TranslatableArg};

use super::Text;

impl Serialize for Text {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut fields = 1; // Content
        if !self.style.is_empty() {
            fields += self.style.count_non_empty_fields();
        }
        if !self.sibs.is_empty() {
            fields += 1;
        }

        #[allow(clippy::single_match)]
        match self.content {
            Content::Translatable(ref val) => {
                if val.fallback().is_some() {
                    fields += 1
                }
                if !val.args().is_empty() {
                    fields += 1
                }
            }
            _ => (),
        }

        let mut state = serializer.serialize_struct("Text", fields)?;

        use serde::ser::SerializeStruct;

        if !self.style.is_empty() {
            macro_rules! serialize_style {
                ($i:expr => $($f:ident),*) => {
                    {
                        use serde::ser::SerializeStruct;
                        $(if let Some(value) = &$i.$f { state.serialize_field(stringify!($f), value)?; })*
                    }
                };
            }

            serialize_style! {
                self.style =>
                color,
                bold,
                italic,
                underlined,
                strikethrough,
                obfuscated,
                click,
                hover,
                insertion,
                font
            }
        }

        if !self.sibs.is_empty() {
            state.serialize_field("extra", &self.sibs)?;
        }

        match self.content {
            Content::Empty => state.serialize_field("text", "")?,
            Content::Literal(ref tc) => state.serialize_field("text", &tc)?,
            Content::Translatable(ref tc) => {
                state.serialize_field("translate", tc.key())?;
                if let Some(fallback) = tc.fallback() {
                    state.serialize_field("fallback", fallback)?;
                }
                if !tc.args().is_empty() {
                    state.serialize_field("with", tc.args())?;
                }
            }
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for Text {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TextVisitor;

        impl<'de> serde::de::Visitor<'de> for TextVisitor {
            type Value = Text;

            #[inline]
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Text")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Text::from(Content::Literal(std::borrow::Cow::Owned(
                    v.to_owned(),
                ))))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Text::from(Content::Literal(std::borrow::Cow::Owned(v))))
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Text::from(Content::Literal(std::borrow::Cow::Borrowed(
                    if v { "true" } else { "false" },
                ))))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Text::from(Content::Literal(std::borrow::Cow::Owned(
                    v.to_string(),
                ))))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Text::from(Content::Literal(std::borrow::Cow::Owned(
                    v.to_string(),
                ))))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Text::from(Content::Literal(std::borrow::Cow::Owned(
                    v.to_string(),
                ))))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut text: Option<Text> = None;

                while let Some(val) = seq.next_element::<Text>()? {
                    if let Some(ref mut t) = text {
                        t.push(val);
                    } else {
                        text = Some(val);
                    }
                }

                Ok(text.unwrap_or_default())
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut m = serde_json::Map::new();
                while let Some((key, value)) = map.next_entry::<String, serde_json::Value>()? {
                    m.insert(key, value);
                }

                use serde::de::Error;
                let mut text: Text;
                if let Some(val) = m.remove("text") {
                    // Literal content
                    text = val.as_str().map_or_else(Default::default, |str| {
                        Content::Literal(std::borrow::Cow::Owned(str.to_owned())).into()
                    })
                } else if let Some(val) = m.remove("translate") {
                    // Translatable content
                    let str;
                    if let serde_json::Value::String(s) = val {
                        str = s
                    } else {
                        return Err(A::Error::custom("expected string for \"translate\" field"));
                    };
                    let fallback_val = m.remove("fallback");
                    if let Some(with_val) = m.remove("with") {
                        let with;
                        if let serde_json::Value::Array(w) = with_val {
                            with = w
                        } else {
                            return Err(A::Error::custom("expected array for \"with\" field"));
                        };

                        let mut args: Vec<TranslatableArg> = Vec::with_capacity(with.len());
                        for obj in with.into_iter().map(serde_json::from_value::<Text>) {
                            args.push(obj.map_err(A::Error::custom)?.into());
                        }
                        text = Content::Translatable(Translatable::new(
                            std::borrow::Cow::Owned(str),
                            if let Some(serde_json::Value::String(s)) = fallback_val {
                                Some(std::borrow::Cow::Owned(s))
                            } else {
                                None
                            },
                            args,
                        ))
                        .into();
                    } else {
                        text = Content::Translatable(Translatable::new(
                            std::borrow::Cow::Owned(str),
                            if let Some(serde_json::Value::String(s)) = fallback_val {
                                Some(std::borrow::Cow::Owned(s))
                            } else {
                                None
                            },
                            vec![],
                        ))
                        .into();
                    }
                } else {
                    return Err(A::Error::custom("don't know kow to turn {m:?} into a Text"));
                }

                if let Some(val) = m.remove("extra") {
                    let mut sibs = Vec::new();
                    if let serde_json::Value::Array(arr) = val {
                        for obj in arr.into_iter().map(serde_json::from_value::<Text>) {
                            sibs.push(obj.map_err(A::Error::custom)?);
                        }
                    } else {
                        return Err(A::Error::custom("expected array for \"extra\" field"));
                    }
                    text.sibs = sibs;
                }

                text.set_style(
                    serde_json::from_value(serde_json::Value::Object(m))
                        .map_err(A::Error::custom)?,
                );

                Ok(text)
            }
        }

        deserializer.deserialize_any(TextVisitor)
    }
}
