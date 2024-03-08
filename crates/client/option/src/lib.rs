//! Minecraft game options

pub(crate) mod callbacks;
pub mod enums;
pub mod tooltip_factory;

use rimecraft_text::Text;

struct SimpleOption<T, Txt, TxtStyle> {
    pub(crate) text: Text<Txt, TxtStyle>,
    pub(crate) value: Option<T>,
    default: T,
    change_callback: fn(T),
}

impl<T, Txt, TxtStyle> SimpleOption<T, Txt, TxtStyle> {
    fn set_value(&mut self, value: Option<T>) {
        todo!()
    }
}
