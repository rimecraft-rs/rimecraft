//! Minecraft client game options.

#[cfg(test)]
mod tests;

pub mod callbacks;

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
    fn apply<'t>(&self, value: &'t V) -> Option<Tooltip<'t, Cx>>;
}

impl<V, Cx, T> TooltipFactory<V, Cx> for T
where
    Cx: ProvideTooltipTy + ProvideTextTy,
    T: Fn(&V) -> Option<Tooltip<'_, Cx>> + ?Sized,
{
    fn apply<'t>(&self, value: &'t V) -> Option<Tooltip<'t, Cx>> {
        (self)(value)
    }
}

/// A [`TooltipFactory`] that always returns [`None`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmptyTooltipFactory;

impl EmptyTooltipFactory {
    /// Creates a new [`EmptyTooltipFactory`].
    pub fn new() -> Self {
        EmptyTooltipFactory
    }
}

impl<V, Cx> TooltipFactory<V, Cx> for EmptyTooltipFactory
where
    Cx: ProvideTooltipTy + ProvideTextTy,
{
    fn apply<'t>(&self, _value: &'t V) -> Option<Tooltip<'t, Cx>> {
        None
    }
}

/// A [`TooltipFactory`] that always returns the same static tooltip.
pub struct StaticTooltipFactory<Cx>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
{
    text: Text<Cx>,
}

impl<Cx> Debug for StaticTooltipFactory<Cx>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
    Cx::Content: Debug,
    Cx::StyleExt: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticTooltipFactory")
            .field("text", &self.text)
            .finish()
    }
}

impl<Cx> StaticTooltipFactory<Cx>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
{
    /// Creates a new [`StaticTooltipFactory`] with the given [`Text`].
    pub fn new(text: Text<Cx>) -> Self {
        Self { text }
    }
}

impl<V, Cx> TooltipFactory<V, Cx> for StaticTooltipFactory<Cx>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
    <Cx as ProvideTextTy>::Content: Clone,
    <Cx as ProvideTextTy>::StyleExt: Clone,
{
    fn apply<'t>(&self, _value: &'t V) -> Option<Tooltip<'t, Cx>> {
        Some(Tooltip::of(self.text.clone()))
    }
}

type DynamicTooltipGetter<'f, V, Cx> = Box<dyn for<'t> Fn(&'t V) -> Option<Tooltip<'t, Cx>> + 'f>;

/// A [`TooltipFactory`] that uses a dynamic factory function.
pub struct DynamicTooltipFactory<'f, V, Cx>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
{
    factory: DynamicTooltipGetter<'f, V, Cx>,
}

impl<V, Cx> Debug for DynamicTooltipFactory<'_, V, Cx>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicTooltipFactory").finish()
    }
}

impl<'f, V, Cx> DynamicTooltipFactory<'f, V, Cx>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
{
    /// Creates a new [`DynamicTooltipFactory`] with the given factory function.
    pub fn new<F>(factory: F) -> Self
    where
        F: for<'t> Fn(&'t V) -> Option<Tooltip<'t, Cx>> + 'f,
    {
        Self {
            factory: Box::new(factory),
        }
    }
}

impl<V, Cx> TooltipFactory<V, Cx> for DynamicTooltipFactory<'_, V, Cx>
where
    Cx: ProvideTooltipTy + ProvideTextTy,
{
    fn apply<'t>(&self, value: &'t V) -> Option<Tooltip<'t, Cx>> {
        (self.factory)(value)
    }
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
    T: Fn(&Text<Cx>, &V) -> Text<Cx> + ?Sized,
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
    pub fn set_value(&mut self, value: V) {
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

/// Creates a boolean [`SimpleOption`] with the specified parameters.
///
/// See: [`callbacks::bool`]
pub fn bool<Cx>(
    text: Text<Cx>,
    value_text_getter: Box<dyn ValueTextGetter<bool, Cx>>,
    default: bool,
    tooltip_factory: Box<dyn TooltipFactory<bool, Cx>>,
    change_callback: Box<dyn Fn(&bool)>,
) -> SimpleOption<bool, Cx>
where
    Cx: ProvideTextTy,
{
    SimpleOption::new(
        text,
        value_text_getter,
        default,
        Box::new(callbacks::bool()),
        tooltip_factory,
        change_callback,
    )
}
