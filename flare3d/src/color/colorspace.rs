use std::fmt::Debug;

pub trait WithAlpha {
    fn alpha(&self) -> f64;
    fn set_alpha(&mut self, alpha: f64);
}

macro_rules! impl_alpha {
    ($t:ty => $f:ident) => {
        impl WithAlpha for $t {
            #[inline]
            fn alpha(&self) -> f64 {
                self.$f
            }

            #[inline]
            fn set_alpha(&mut self, alpha: f64) {
                self.$f = alpha
            }
        }
    };
}

macro_rules! impl_channel {
    ($t:ty => $raw:ident, $($i:literal => ($channel:ident, $set_channel:ident)),*) => {
        $(
			impl $t {
                #[inline]
                pub fn $channel(&self) -> f64 {
                    self.$raw[$i]
                }

                #[inline]
                pub fn $set_channel(&mut self, channel: f64) {
                    self.$raw[$i] = channel
                }
			}
        )*
    };
}

// Colorspaces

#[derive(Debug)]
pub struct RGBColor {
    alpha: f64,
    raw: [f64; 3],
}

impl RGBColor {
	pub fn new(alpha: f64, red: f64, green: f64, blue: f64) -> RGBColor {
		RGBColor { alpha: alpha, raw: [red, green, blue] }
	}
}

impl_alpha!(RGBColor => alpha);
impl_channel!(RGBColor => raw, 0 => (red, set_red), 1 => (green, set_green), 2 => (blue, set_blue));

#[derive(Debug)]
pub struct HSVColor {
    alpha: f64,
    raw: [f64; 3],
}

impl_alpha!(HSVColor => alpha);
impl_channel!(HSVColor => raw, 0 => (hue, set_hue), 1 => (saturation, set_saturation), 2 => (value, set_value));

#[derive(Debug)]
pub struct HSLColor {
    alpha: f64,
    raw: [f64; 3],
}

impl_alpha!(HSLColor => alpha);
impl_channel!(HSLColor => raw, 0 => (hue, set_hue), 1 => (saturation, set_saturation), 2 => (lightness, set_lightness));

#[derive(Debug)]
pub struct CMYKColor {
    alpha: f64,
    raw: [f64; 4],
}

impl_alpha!(CMYKColor => alpha);
impl_channel!(CMYKColor => raw, 0 => (cyan, set_cyan), 1 => (magenta, set_mamgenta), 2 => (yellow, set_yellow), 3 => (black, set_black));

#[derive(Debug)]
pub struct XYZColor {
    alpha: f64,
    raw: [f64; 3],
}

impl_alpha!(XYZColor => alpha);
impl_channel!(XYZColor => raw, 0 => (x, set_x), 1 => (y, set_y), 2 => (z, set_z));

#[derive(Debug)]
pub struct LabColor {
    alpha: f64,
    raw: [f64; 3],
}

impl_alpha!(LabColor => alpha);
impl_channel!(LabColor => raw, 0 => (lightness, set_lightness), 1 => (a, set_a), 2 => (b, set_b));

#[derive(Debug)]
pub struct LChColor {
    alpha: f64,
    raw: [f64; 3],
}

impl_alpha!(LChColor => alpha);
impl_channel!(LChColor => raw, 0 => (luminance, set_luminance), 1 => (chroma, set_chroma), 2 => (hue, set_hue));

// Convertions

type Intermediate = RGBColor;

macro_rules! intermediate_impl {
    ($t:ty) => {
        impl<T: Into<Intermediate>> From<T> for $t {
            fn from(value: T) -> Self {
                Intermediate::from(value).into()
            }
        }
    };
}
