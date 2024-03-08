//! Minecraft game options

pub(crate) mod callbacks;
pub mod enums;
pub mod tooltip_factory;

use rimecraft_text::{ProvideTextTy, Text};

struct SimpleOption<T, Cx>
where
    Cx: ProvideTextTy,
{
    pub(crate) text: Text<Cx>,
    pub(crate) value: Option<T>,
    default: T,
    change_callback: fn(T),
}

impl<T, Txt> SimpleOption<T, Txt>
where
    Txt: ProvideTextTy,
{
    fn set_value(&mut self, value: Option<T>) {
        todo!()
    }
}
