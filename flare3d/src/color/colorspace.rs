use std::fmt::Debug;

pub trait WithAlpha {
	fn alpha(&self) -> f64;
	fn set_alpha(&mut self, alpha: f64);
}

macro_rules! impl_alpha {
    ($t:ty => $f:tt) => {
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
    ($t:ty, $raw:tt => $($i:literal, $channel:tt),*) => {
        impl $t {
            $(
                #[inline]
                pub fn $channel(&self) -> f64 {
                    self.$raw[$i]
                }
            )*

            $(
                #[inline]
                pub fn set_$channel(&mut self, channel: f64) {
                    self.$raw[$i] = channel;
                }
            )*
        }
    };
}

// Colorspaces

#[derive(Debug)]
pub struct RGBColor {
	alpha: f64,
	raw: [f64; 3],
}

impl_alpha!(RGBColor => alpha);
impl_channel!(RGBColor, raw => 0, red, 1, green, 2, blue);

#[derive(Debug)]
pub struct HSVColor {
	alpha: f64,
	raw: [f64; 3],
}

impl HSVColor {
	pub fn hue(&self) -> f64 {
		self.raw[0]
	}

	pub fn saturation(&self) -> f64 {
		self.raw[1]
	}

	pub fn value(&self) -> f64 {
		self.raw[2]
	}
}

#[derive(Debug)]
pub struct HSLColor {
	alpha: f64,
	raw: [f64; 3],
}

impl HSLColor {
	pub fn hue(&self) -> f64 {
		self.raw[0]
	}

	pub fn saturation(&self) -> f64 {
		self.raw[1]
	}

	pub fn lightness(&self) -> f64 {
		self.raw[2]
	}
}

#[derive(Debug)]
pub struct CMYKColor {
	alpha: f64,
	raw: [f64; 4],
}

impl CMYKColor {
	pub fn cyan(&self) -> f64 {
		self.raw[0]
	}

	pub fn magenta(&self) -> f64 {
		self.raw[1]
	}

	pub fn yellow(&self) -> f64 {
		self.raw[2]
	}

	pub fn black(&self) -> f64 {
		self.raw[3]
	}
}

#[derive(Debug)]
pub struct XYZColor {
	alpha: f64,
	raw: [f64; 3],
}

impl XYZColor {
	pub fn x(&self) -> f64 {
		self.raw[0]
	}

	pub fn y(&self) -> f64 {
		self.raw[1]
	}

	pub fn z(&self) -> f64 {
		self.raw[2]
	}
}

#[derive(Debug)]
pub struct LabColor {
	alpha: f64,
	raw: [f64; 3],
}

impl LabColor {
	pub fn lightness(&self) -> f64 {
		self.raw[0]
	}

	pub fn a(&self) -> f64 {
		self.raw[1]
	}

	pub fn b(&self) -> f64 {
		self.raw[2]
	}
}

#[derive(Debug)]
pub struct LChColor {
	alpha: f64,
	raw: [f64; 3],
}

impl LChColor {
	pub fn luminance(&self) -> f64 {
		self.raw[0]
	}

	pub fn chroma(&self) -> f64 {
		self.raw[1]
	}

	pub fn hue(&self) -> f64 {
		self.raw[2]
	}
}

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
