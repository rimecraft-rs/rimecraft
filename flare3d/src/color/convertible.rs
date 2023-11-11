use super::{colorspace::Colorspace, mix_mode::MixMode, Channels};

pub trait Convertible {
	fn from(&self, channels: Channels, colorspace: Colorspace) -> Channels;

	fn to(&self, channels: Channels, colorspace: Colorspace) -> Channels;

	fn mix(&self, channels: Channels, another: Channels, ratio: f64, colorspace: Colorspace, mix_mode: MixMode);
}