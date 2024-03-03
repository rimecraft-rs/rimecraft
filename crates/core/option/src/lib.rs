pub(crate) mod callbacks;
pub mod tooltip_factory;

use rimecraft_text::Text;

struct SimpleOption<T> {
	pub(crate) text_getter: fn(T) -> Text<(), ()>,
	pub(crate) text: Text<(), ()>,
	pub(crate) value: Option<T>,
	default: T,
	change_callback: fn(T),
}

impl<T> SimpleOption<T> {
	fn set_value(&mut self, value: Option<T>) {
		todo!()
	}
}