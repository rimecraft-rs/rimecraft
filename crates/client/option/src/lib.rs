//! Minecraft game options

pub(crate) mod callbacks;
pub mod enums;

use std::fmt::Debug;

use callbacks::Callbacks;
use rimecraft_text::Text;

/// Provides a tooltip.
pub trait TooltipFactory<T> {
    fn apply(&self, value: T) -> Option<()>; // Option<Tooltip>
}

/// Represents a simple option, containing a value and a default value.
pub struct SimpleOption<T, Txt, TxtStyle> {
    pub(crate) text: Text<Txt, TxtStyle>,
    pub(crate) value: Option<T>,
    default: T,
    change_callback: fn(&T),
    callbacks: Box<dyn Callbacks<T, Txt, TxtStyle>>,
    tooltip_factory: Box<dyn TooltipFactory<T>>,
}

impl<T, Txt, TxtStyle> SimpleOption<T, Txt, TxtStyle>
where
    T: Clone + PartialEq,
{
	pub fn get_value(&self) -> T {
		match self.value.clone() {
			Some(value) => value,
			None => self.default.clone(),
		}
	}

    pub fn set_value(&mut self, value: Option<T>) {
        let value = self
            .callbacks
            .validate(value)
            .unwrap_or(self.default.clone());
        if todo!()
        /* Checks if Minecraft client isn't running */
        {
            self.value = Some(value);
        } else {
			if value != self.get_value() {
				self.value = Some(value);
				(self.change_callback)(&value);
			}
        }
    }

    pub fn to_string(&self) -> String {
        todo!()
    }
}

//TODO: cargo fmt
impl<T, Txt, TxtStyle> Debug for SimpleOption<T, Txt, TxtStyle>
where
    T: Debug,
    Txt: Debug,
    TxtStyle: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleOption")
            .field("text", &self.text)
            .field("value", &self.value)
            .field("default", &self.default)
            .finish()
    }
}
