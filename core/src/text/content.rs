use parking_lot::RwLock;

use std::{
    borrow::Cow,
    sync::{Arc, Weak},
};

use crate::lang::DebugLang;

use super::{
    visit::{StyledVisit, Visit},
    Style, Text,
};

#[deprecated]
pub trait ContentDeprecated: Visit<()> + StyledVisit<Style> {}

#[derive(Debug, Clone)]
pub enum Content {
    Empty,
    Literal(Cow<'static, str>),
    Translatable(Translatable),
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

#[derive(Debug)]
pub struct Translatable {
    key: Cow<'static, String>,
    fallback: Option<Cow<'static, str>>,
    lang: Weak<dyn DebugLang>,
}

impl Translatable {
    fn translations(&self) -> Vec<Text> {}
}
