//! Minecraft client game options.

pub mod callbacks;
pub mod content;

use rimecraft_client_tooltip::{ProvideTooltipTy, Tooltip};
use rimecraft_text::{ProvideTextTy, Text};
use std::fmt::Debug;

use crate::callbacks::Callbacks;

pub trait TooltipFactory<V, Cx>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
{
    fn apply(&self, value: V) -> Option<Tooltip<Cx>>;
}

impl<V, Cx, T> TooltipFactory<V, Cx> for T
where
    Cx: ProvideTooltipTy + ProvideTextTy,
    T: Fn(V) -> Option<Tooltip<Cx>>,
{
    fn apply(&self, value: V) -> Option<Tooltip<Cx>> {
        (self)(value)
    }
}

pub trait ValueTextGetter<V, Cx>
where
    Cx: ProvideTextTy,
{
    fn get_value_text(&self, option_text: &Text<Cx>, value: &V) -> Text<Cx>;
}

impl<V, Cx, T> ValueTextGetter<V, Cx> for T
where
    Cx: ProvideTextTy,
    T: Fn(&Text<Cx>, &V) -> Text<Cx>,
{
    fn get_value_text(&self, option_text: &Text<Cx>, value: &V) -> Text<Cx> {
        (self)(option_text, value)
    }
}

pub struct SimpleOption<V, Cx>
where
    Cx: ProvideTextTy,
{
    pub(crate) text: Text<Cx>,
    pub(crate) value_text_getter: Box<dyn ValueTextGetter<V, Cx>>,
    pub(crate) value: V,
    default: V,
    callbacks: Box<dyn Callbacks<V, Cx>>,
    tooltip_factory: Box<dyn TooltipFactory<V, Cx>>,
    change_callback: Box<dyn Fn(Option<V>)>,
}

impl<V, Cx> SimpleOption<V, Cx>
where
    Cx: ProvideTextTy,
    V: Clone,
{
    pub fn new(
        text: Text<Cx>,
        value_text_getter: Box<dyn ValueTextGetter<V, Cx>>,
        default: V,
        callbacks: Box<dyn Callbacks<V, Cx>>,
        tooltip_factory: Box<dyn TooltipFactory<V, Cx>>,
        change_callback: Box<dyn Fn(Option<V>)>,
    ) -> Self {
        Self {
            text,
            value_text_getter,
            value: default.clone(),
            default,
            callbacks,
            tooltip_factory,
            change_callback,
        }
    }
}

impl<V, Cx> Debug for SimpleOption<V, Cx>
where
    V: Debug,
    Cx: ProvideTextTy,
    Cx::Content: Debug,
    Cx::StyleExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleOption")
            .field("text", &self.text)
            .field("value", &self.value)
            .field("default", &self.default)
            .finish()
    }
}

impl<V, Cx> SimpleOption<V, Cx>
where
    Cx: ProvideTextTy,
{
    pub fn set_value(&mut self, value: &V) {
        todo!()
    }
}
