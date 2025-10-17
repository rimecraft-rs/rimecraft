//! Minecraft client game options.

pub(crate) mod callbacks;
pub mod content;
pub mod tooltip_factory;

use rimecraft_text::{ProvideTextTy, Text};
use std::fmt::Debug;

pub type ChangeCallback<T> = dyn Fn(Option<T>);

pub struct SimpleOption<T, Txt>
where
    Txt: ProvideTextTy,
{
    pub(crate) text: Text<Txt>,
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

impl<T, Txt> SimpleOption<T, Txt>
where
    Txt: ProvideTextTy,
{
    pub fn set_value(&mut self, value: Option<T>) {
        todo!()
    }
}
