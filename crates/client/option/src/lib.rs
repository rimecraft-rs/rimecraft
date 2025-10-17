//! Minecraft client game options.

pub mod callbacks;
pub mod content;
pub mod tooltip_factory;

use rimecraft_text::{ProvideTextTy, Text};
use std::fmt::Debug;

pub type ChangeCallback<T> = dyn Fn(Option<T>);

pub struct SimpleOption<T, Cx>
where
    Cx: ProvideTextTy,
{
    pub(crate) text: Text<Cx>,
    pub(crate) value: Option<T>,
    default: T,
    change_callback: Box<ChangeCallback<T>>,
}

impl<Cx> Debug for SimpleOption<(), Cx>
where
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

impl<T, Cx> SimpleOption<T, Cx>
where
    Cx: ProvideTextTy,
{
    pub fn set_value(&mut self, value: Option<T>) {
        todo!()
    }
}
