use parking_lot::RwLock;
use lazy_regex::Captures;

use std::{
    any::Any,
    borrow::Cow,
    sync::{Arc, Weak},
};

use crate::lang::{DebugLang, LangDepended, LangExt, UpdateLang};

use super::{
    visit::{self, StyledVisit, Visit},
    Style, Text,
};

#[deprecated]
pub trait ContentDeprecated: Visit<()> + StyledVisit<Style> {}

#[derive(Debug, Clone, Hash)]
pub enum Content {
    Empty,
    Literal(Cow<'static, str>),
    Translatable(()),
}

impl<T> Visit<T> for Content {
    fn visit<V: super::visit::Visitor<T> + ?Sized>(&self, visitor: &mut V) -> Option<T> {
        match self {
            Content::Literal(value) => visitor.accept(value),
            Content::Translatable(_) => todo!(),
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
            Content::Translatable(_) => todo!(),
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
}

#[derive(Debug, thiserror::Error)]
pub enum TranslateParseError {
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
}

#[derive(Debug)]
pub struct Translatable {
    inner: LangDepended<TranslatableInner>,
}

impl Translatable {}

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

#[derive(Debug, Clone)]
struct TranslatableInner {
    parts: Vec<VisitAbst>,
    key: Cow<'static, String>,
    fallback: Option<Cow<'static, str>>,

    args: Vec<Arc<dyn Any + Sync + Send>>,
}

static T_ARG_FMT: lazy_regex::Lazy<lazy_regex::Regex> =
    lazy_regex::lazy_regex!("%(?:(\\d+)\\$)?([A-Za-z%]|$)");

impl TranslatableInner {
    fn update_parts(&mut self, translation: &str) -> Result<(), TranslationError> {
        let mut last_end = 0;
        
        for (g,m) in T_ARG_FMT.captures_iter(translation).map(|e|e.extract()) {
            if last_end != m.start() {
                self.parts.push(VisitAbst::Plain(visit::plain(Cow::Owned(
                    translation[last_end..m.start()].to_owned(),
                ))));
            }
            self.parts.push(VisitAbst::Plain(visit::plain(Cow::Owned(
                m.as_str().to_owned(),
            ))));
            last_end = m.end();
        }
        Ok(())
    }
}

impl UpdateLang for TranslatableInner {
    fn update_from_lang(&mut self, lang: &dyn DebugLang) {
        let translation = self.fallback.as_ref().map_or_else(
            || lang.translation_or_key(&self.key),
            |fallback| lang.translation(&self.key).unwrap_or(&fallback),
        );
        self.parts.clear();
    }
}
