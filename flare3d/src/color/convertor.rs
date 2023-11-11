use super::{colorspace::Colorspace, Channels, Channel};

fn strip_prefix(color: &str) -> &str {
	color.strip_prefix("0x").unwrap_or(color.strip_prefix("#").unwrap_or(color))
}

pub fn int_to_hex(color: usize) -> String {
	format!("0x{:0>6X}", color & 0xFFFFFF)
}

pub fn hex_to_int(color: &str) -> usize {
	usize::from_str_radix(strip_prefix(color), 16).unwrap()
}

/// Clamps a color channel into the range of `[0.0, 1.0]`. Inputs are the type of the channel and the channel value as an expression.
///
/// # Examples
///
/// ```
/// assert_eq!(1.0, clamp_channel!(f64: 5.0));
/// assert_eq!(0, clamp_channel!(i32: -2));
/// assert_eq!(0.2, clamp_channel!(f32: 0.2));
/// ```
macro_rules! clamp_channel {
	($t: ty: $c: expr) => {
		<$t>::max(0 as $t, <$t>::min(1 as $t, $c))
	}
}

pub fn validate(channels: Channels) -> Channels {
	std::array::from_fn(|i| clamp_channel!(Channel: channels[i]))
}

pub fn to_rgba(rgba: Channels, colorspace: Colorspace) -> Channels {
	todo!()
}