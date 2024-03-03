//! Minecraft game options

pub(crate) mod callbacks;
pub mod tooltip_factory;
pub mod enums;

use rimecraft_text::{Text, Texts};

struct SimpleOption<T, Txt>
where
    Txt: Texts,
{
    pub(crate) text: Text<Txt::T, Txt::StyleExt>,
    pub(crate) value: Option<T>,
    default: T,
    change_callback: fn(T),
}

impl<T, Txt> SimpleOption<T, Txt>
where
    Txt: Texts,
{
    fn set_value(&mut self, value: Option<T>) {
        todo!()
    }
}
