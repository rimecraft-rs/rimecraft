//! Minecraft client game options.

pub mod callbacks;
pub mod content;

use rimecraft_client_tooltip::{ProvideTooltipTy, Tooltip};
use rimecraft_text::{ProvideTextTy, Text};
use std::fmt::{Debug, Display};

use crate::callbacks::Callbacks;

/// A factory for creating [`Tooltip`]s based on a value.
pub trait TooltipFactory<V, Cx>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
{
    /// Creates a [`Tooltip`] for the given value, or [`None`] if no tooltip should be shown.
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

/// Creates an empty tooltip factory that always returns [`None`].
pub fn empty_tooltip_factory<V, Cx>() -> Box<dyn TooltipFactory<V, Cx>>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
{
    Box::new(|_value: V| None)
}

/// Creates a static tooltip factory that always returns the given [`Text`] as tooltip.
pub fn static_tooltip_factory<V, Cx>(text: Text<Cx>) -> Box<dyn TooltipFactory<V, Cx>>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
    <Cx as ProvideTextTy>::Content: Clone,
    <Cx as ProvideTextTy>::StyleExt: Clone,
{
    Box::new(move |_value: V| Some(Tooltip::of(text.clone())))
}

/// A trait for getting the display text for a value.
pub trait ValueTextGetter<V, Cx>
where
    Cx: ProvideTextTy,
{
    /// Gets the display text for the given value.
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

/// A simple implementation of an option with a value of type `V`.
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
    change_callback: Box<dyn Fn(&V)>,
}

impl<V, Cx> SimpleOption<V, Cx>
where
    Cx: ProvideTextTy,
    V: Clone,
{
    /// Creates a new [`SimpleOption`].
    pub fn new(
        text: Text<Cx>,
        value_text_getter: Box<dyn ValueTextGetter<V, Cx>>,
        default: V,
        callbacks: Box<dyn Callbacks<V, Cx>>,
        tooltip_factory: Box<dyn TooltipFactory<V, Cx>>,
        change_callback: Box<dyn Fn(&V)>,
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

impl<V, Cx> Display for SimpleOption<V, Cx>
where
    Cx: ProvideTextTy,
    Cx::Content: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl<V, Cx> SimpleOption<V, Cx>
where
    Cx: ProvideTextTy,
    V: Clone + PartialEq,
{
    /// Sets the value of the option, invoking the change callback if the value changes.
    pub fn set_value(&mut self, value: &V) {
        let value = self
            .callbacks
            .validate(value)
            .unwrap_or_else(|| self.default.clone());
        if value != self.value {
            self.value = value;
            (self.change_callback)(&self.value);
        }
    }

    /// Gets the [`Callbacks`] of the option.
    pub fn get_callbacks(&self) -> &dyn Callbacks<V, Cx> {
        &*self.callbacks
    }
}
