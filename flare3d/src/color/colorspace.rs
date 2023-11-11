use std::fmt::Display;

use super::{Channel, convertible::Convertible};

/// Supported colorspaces:
/// - **RGB**							- red, green, blue;
/// - **HSV**							- hue, saturation, value;
/// - **HSL**							- hue, saturation, lightness;
/// - **CMYK**							- cyan, magenta, yellow, black;
/// - **XYZ**							- x-axis, y-axis, z-axis;
/// - **Lab *(L\*a\*b\* or CIELAB)***	- Lightness, green-magenta, blue-yellow;
/// - **LCh *(HCL or CIELCh)***			- Luminance, Chroma, hue.
#[derive(Debug)]
pub enum Colorspace {
    RGB,
    HSV,
    HSL,
    CMYK,
    XYZ,
    Lab,
    LCh,
}

impl Colorspace {
	pub fn name(&self) -> &'static str {
		match self {
            Colorspace::RGB => "RGB",
            Colorspace::HSV => "HSV",
            Colorspace::HSL => "HSL",
            Colorspace::CMYK => "CMYK",
            Colorspace::XYZ => "XYZ",
            Colorspace::Lab => "L*a*b*",
            Colorspace::LCh => "LCh",
        }
	}
}

impl Display for Colorspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name())
    }
}

/*
impl Convertible for Colorspace {
    fn from(&self, channels: &[Channel], colorspace: Colorspace) {
        match self {
			
		}
    }

    fn to(&self, channels: &[Channel], colorspace: Colorspace) {
        todo!()
    }

    fn mix(&self, channels: &[Channel], another: &[Channel], ratio: f64, colorspace: Colorspace, mix_mode: super::mix_mode::MixMode) {
        todo!()
    }
}
*/
