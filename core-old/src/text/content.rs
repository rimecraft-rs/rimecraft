use serde::Serialize;

use std::{borrow::Cow, fmt::Debug, sync::Arc};

use crate::lang::{DebugLang, LangDepended, LangExt, UpdateLang};

use super::{
    visit::{self, StyledVisit, Visit},
    Style, Text,
};

#[derive(Debug, Clone, Default)]
pub enum Content {
    #[default]
    Empty,
    Literal(Cow<'static, str>),
    Translatable(Translatable),
}

impl<T> Visit<T> for Content {
    fn visit<V: super::visit::Visitor<T> + ?Sized>(&self, visitor: &mut V) -> Option<T> {
        match self {
            Content::Literal(value) => visitor.accept(value),
            Content::Translatable(value) => value.visit(visitor),
            _ => None,
        }
    }
}

impl<T> StyledVisit<T> for Content {
    fn styled_visit<V: super::visit::StyleVisitor<T> + ?Sized>(
        &self,
        visitor: &mut V,
        style: &Style,
    ) -> Option<T> {
        match self {
            Content::Literal(value) => visitor.accept(style, value),
            Content::Translatable(value) => value.styled_visit(visitor, style),
            _ => None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("error parsing {0}: {1}")]
    Parse(Cow<'static, str>, TranslateParseError),
    #[error("invalid index {1} requested for {0}")]
    Index(Cow<'static, str>, usize),
    #[error("illegal argument: {0}")]
    IllegalArg(String),
}

#[derive(Debug, thiserror::Error)]
pub enum TranslateParseError {
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("error parsing integer: {0}")]
    ParseInt(std::num::ParseIntError),
}

#[derive(Debug, Clone)]
pub struct Translatable {
    cx: TranslatableContext,
    parts: LangDepended<TranslatableParts>,
}

impl Translatable {
    pub fn new(
        key: Cow<'static, str>,
        fallback: Option<Cow<'static, str>>,
        args: Vec<TranslatableArg>,
    ) -> Self {
        Self {
            cx: TranslatableContext {
                key,
                fallback,
                args,
            },
            parts: LangDepended::new(TranslatableParts { parts: vec![] }),
        }
    }

    #[inline]
    pub fn fallback(&self) -> Option<&str> {
        self.cx.fallback.as_deref()
    }

    #[inline]
    pub fn key(&self) -> &str {
        &self.cx.key
    }

    #[inline]
    pub fn args(&self) -> &[TranslatableArg] {
        &self.cx.args
    }
}

#[derive(Debug, Clone)]
struct TranslatableContext {
    key: Cow<'static, str>,
    fallback: Option<Cow<'static, str>>,
    args: Vec<TranslatableArg>,
}

impl TranslatableContext {
    fn arg(&self, index: usize) -> Option<VisitAbst> {
        self.args.get(index).map(|e| match e {
            TranslatableArg::Text(text) => VisitAbst::Text(text.clone()),
            TranslatableArg::Display(disp) => {
                VisitAbst::Plain(visit::plain(Cow::Owned(disp.to_string())))
            }
            TranslatableArg::Debug(deb) => {
                VisitAbst::Plain(visit::plain(Cow::Owned(format!("{deb:?}"))))
            }
        })
    }
}

impl<T> visit::Visit<T> for Translatable {
    fn visit<V: visit::Visitor<T> + ?Sized>(&self, visitor: &mut V) -> Option<T> {
        self.parts
            .get(&self.cx)
            .parts
            .iter()
            .find_map(|val| visit::Visit::visit(val, visitor))
    }
}

impl<T> visit::StyledVisit<T> for Translatable {
    fn styled_visit<V: visit::StyleVisitor<T> + ?Sized>(
        &self,
        visitor: &mut V,
        style: &Style,
    ) -> Option<T> {
        self.parts
            .get(&self.cx)
            .parts
            .iter()
            .find_map(|val| visit::StyledVisit::styled_visit(val, visitor, style))
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
enum VisitAbst {
    Plain(visit::Plain<'static>),
    Styled(visit::Styled<'static>),
    Text(Text),
}

impl<T> visit::Visit<T> for VisitAbst {
    fn visit<V: visit::Visitor<T> + ?Sized>(&self, visitor: &mut V) -> Option<T> {
        match self {
            VisitAbst::Plain(value) => value.visit(visitor),
            VisitAbst::Styled(value) => value.visit(visitor),
            VisitAbst::Text(value) => value.visit(visitor),
        }
    }
}

impl<T> visit::StyledVisit<T> for VisitAbst {
    fn styled_visit<V: visit::StyleVisitor<T> + ?Sized>(
        &self,
        visitor: &mut V,
        style: &Style,
    ) -> Option<T> {
        match self {
            VisitAbst::Plain(value) => value.styled_visit(visitor, style),
            VisitAbst::Styled(value) => value.styled_visit(visitor, style),
            VisitAbst::Text(value) => value.styled_visit(visitor, style),
        }
    }
}

#[derive(Clone)]
pub enum TranslatableArg {
    Text(Text),
    Display(Arc<dyn std::fmt::Display + Send + Sync>),
    Debug(Arc<dyn std::fmt::Debug + Send + Sync>),
}

impl Debug for TranslatableArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(arg0) => f.debug_tuple("Text").field(arg0).finish(),
            Self::Display(arg0) => f.debug_tuple("Display").field(&arg0.to_string()).finish(),
            Self::Debug(arg0) => f.debug_tuple("Debug").field(arg0).finish(),
        }
    }
}

impl From<Text> for TranslatableArg {
    #[inline]
    fn from(text: Text) -> Self {
        if text.style.is_empty() && text.sibs.is_empty() {
            if let Content::Literal(ref lit) = text.content {
                return TranslatableArg::Display(Arc::new(lit.clone()));
            }
        }
        TranslatableArg::Text(text)
    }
}

impl Serialize for TranslatableArg {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Text(arg0) => arg0.serialize(serializer),
            Self::Display(arg0) => serializer.serialize_str(&arg0.to_string()),
            Self::Debug(arg0) => serializer.serialize_str(&format!("{:?}", arg0)),
        }
    }
}

#[derive(Debug, Clone)]
struct TranslatableParts {
    parts: Vec<VisitAbst>,
}

static T_ARG_FMT: lazy_regex::Lazy<lazy_regex::Regex> =
    lazy_regex::lazy_regex!("%(?:(\\d+)\\$)?([A-Za-z%]|$)");

impl TranslatableParts {
    fn update_parts(
        &mut self,
        translation: &str,
        cx: &TranslatableContext,
    ) -> Result<(), TranslationError> {
        macro_rules! validate_arg {
            ($a:ident) => {
                if $a.find('%').is_some() {
                    return Err(TranslationError::IllegalArg($a.to_string()));
                }
            };
        }

        let mut last_end = 0;
        let mut i = 0;

        for (g, m) in T_ARG_FMT
            .captures_iter(translation)
            .zip(T_ARG_FMT.find_iter(translation))
        {
            if last_end != m.start() {
                let str = &translation[last_end..m.start()];
                validate_arg!(str);
                self.parts
                    .push(VisitAbst::Plain(visit::plain(Cow::Owned(str.to_owned()))));
            }

            match (g.get(2).map(|e| e.as_str()).unwrap_or_default(), m.as_str()) {
                ("%", "%%") => {
                    const LIT_PERC_SIGN: visit::Plain<'static> = visit::plain(Cow::Borrowed("%"));
                    self.parts.push(VisitAbst::Plain(LIT_PERC_SIGN));
                }
                ("s", _) => {
                    if let Some(arg) = cx.arg(
                        g.get(1)
                            .map_or_else(
                                || {
                                    let ii = i;
                                    i += 1;
                                    Ok(ii)
                                },
                                |e| e.as_str().parse().map(|ee: usize| ee - 1),
                            )
                            .map_err(|err| {
                                TranslationError::Parse(
                                    cx.key.clone(),
                                    TranslateParseError::ParseInt(err),
                                )
                            })?,
                    ) {
                        self.parts.push(arg);
                    }
                }
                (_, fmt) => {
                    return Err(TranslationError::Parse(
                        cx.key.clone(),
                        TranslateParseError::UnsupportedFormat(fmt.to_owned()),
                    ))
                }
            }

            if last_end < translation.len() {
                let str = &translation[last_end..];
                validate_arg!(str);
                self.parts
                    .push(VisitAbst::Plain(visit::plain(Cow::Owned(str.to_owned()))));
            }

            last_end = m.end();
        }
        Ok(())
    }
}

impl UpdateLang for TranslatableParts {
    type Context = TranslatableContext;

    fn update_from_lang(&mut self, lang: &dyn DebugLang, cx: &Self::Context) {
        let translation = cx
            .fallback
            .as_ref()
            .map_or_else(
                || lang.translation_or_key(&cx.key),
                |fallback| lang.translation(&cx.key).unwrap_or(fallback),
            )
            .to_owned();
        self.parts.clear();

        if self.update_parts(&translation, cx).is_err() {
            self.parts
                .push(VisitAbst::Plain(visit::plain(Cow::Owned(translation))));
        }
    }
}
