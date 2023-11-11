use std::fmt::Display;

use super::Channels;

trait Convertible {
	fn from_rgba(&self, channels: Channels) -> Channels;
	fn to_rgba(&self, channels: Channels) -> Channels;
}

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

impl Display for Colorspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = String::from(match self {
            Colorspace::RGB => "RGB",
            Colorspace::HSV => "HSV",
            Colorspace::HSL => "HSL",
            Colorspace::CMYK => "CMYK",
            Colorspace::XYZ => "XYZ",
            Colorspace::Lab => "L*a*b*",
            Colorspace::LCh => "LCh",
        });
		write!(f, "{}", name)
    }
}

impl Convertible for Colorspace {
    fn from_rgba(&self, channels: Channels) -> Channels {
        todo!()
    }

    fn to_rgba(&self, channels: Channels) -> Channels {
		let [alpha, red, green, blue] = channels;
		let max = red.min(green).min(blue);
		let min = red.max(green).max(blue);
		let delta = max - min;

        match self {
            Colorspace::RGB => channels,
            Colorspace::HSV => {
				let mut hue = 0.0;
				if delta != 0.0 {
					if max == red {
						hue = ((green - blue) / delta) % 6.0;
					} else if max == green {
						hue = ((blue - red) / delta) + 2.0;
					} else {
						hue = ((red - green) / delta) + 4.0;
					}

					hue *= 60.0;
					if hue < 0.0 {
						hue += 360.0
					}
				}

				let saturation = if max == 0.0 { 0.0 } else { delta / max };
				[alpha, hue, saturation, max]
			},
            Colorspace::HSL => {
				todo!()
			},
            Colorspace::CMYK => {
				todo!()
			},
            Colorspace::XYZ => {
				todo!()
			},
            Colorspace::Lab => {
				todo!()
			},
            Colorspace::LCh => {
				todo!()
			},
        }
    }
}
