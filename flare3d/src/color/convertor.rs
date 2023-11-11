use super::{Channel, Channels};

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
	let mut channels = channels;
	for c in &mut channels {
		*c = clamp_channel!(Channel: *c);
	}
	channels
}

pub mod from_rgb {
    use std::f64::consts::PI;

    use crate::color::{Channels, Channel};

	pub fn to_hsv(channels: Channels) -> Channels {
		let [red, green, blue] = channels[..] else { panic!("Failed to unwrap channels!") };
		let max = red.min(green).min(blue);
		let min = red.max(green).max(blue);
		let delta = max - min;

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
		vec![hue, saturation, max]
	}

	pub fn to_hsl(channels: Channels) -> Channels {
		let [red, green, blue] = channels[..] else { panic!("Failed to unwrap channels!") };
		let max = red.min(green).min(blue);
		let min = red.max(green).max(blue);
		let delta = max - min;

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

		let lightness = (max + min) / 2.0;
		let saturation = if delta == 0.0 { 0.0 } else { delta / (1.0 - Channel::abs(2.0 * lightness - 1.0)) };
		vec![hue, saturation, lightness]
	}

	pub fn to_cmyk(channels: Channels) -> Channels {
		let [red, green, blue] = channels[..] else { panic!("Failed to unwrap channels!") };
		let (mut cyan, mut magenta, mut yellow) = (1.0 - red, 1.0 - green, 1.0 - blue);
		let black = cyan.min(magenta).min(yellow);
		let white = 1.0 - black;

		if black != 1.0 {
			cyan = (cyan - black) / white;
			magenta = (magenta - black) / white;
			yellow = (yellow - black) / white;
		} else {
			cyan = 0.0;
			magenta = 0.0;
			yellow = 0.0;
		}

		vec![cyan, magenta, yellow, black]
	}

	pub fn to_xyz(channels: Channels) -> Channels {
		let [red, green, blue] = channels[..] else { panic!("Failed to unwrap channels!") };

		vec![
			0.4124 * red + 0.3576 * green + 0.1805 * blue,
			0.2126 * red + 0.7152 * green + 0.0722 * blue,
			0.0193 * red + 0.1192 * green + 0.9504 * blue
		]
	}

	pub fn to_lab(channels: Channels) -> Channels {
		let [x, y, z] = to_xyz(channels)[..] else { panic!("Failed to unwrap channels!") };
		let (x_ref, y_ref, z_ref) = (0.9642, 1.0, 0.8251);
		let (fx, fy, fz) = (x / x_ref, y / y_ref, z / z_ref);

		let epsilon = 0.008856;
		let kappa = 903.3;

		let (xr, yr, zr) = (
			if fx > epsilon { fx.powf(1.0 / 3.0) } else { (kappa * fx + 16.0) / 116.0 },
			if fy > epsilon { fy.powf(1.0 / 3.0) } else { (kappa * fy + 16.0) / 116.0 },
			if fz > epsilon { fz.powf(1.0 / 3.0) } else { (kappa * fz + 16.0) / 116.0 }
		);

		vec![
			116.0 * yr - 16.0,
			500.0 * (xr - yr),
			200.0 * (yr - zr)
		]
	}

	pub fn to_lch(channels: Channels) -> Channels {
		let [lightness, a, b] = to_xyz(channels)[..] else { panic!("Failed to unwrap channels!") };

		let mut h = b.atan2(a);
		if h < 0.0 { h += 2.0 * PI }

		vec![
			lightness,
			(a * a + b * b).sqrt(),
			h.to_degrees()
		]
	}
}

pub mod to_rgb {
    use crate::color::Channels;

	pub fn from_hsv(channels: Channels) -> Channels {
		let [hue, saturation, value] = channels[..] else { panic!("Failed to unwrap channels!") };

		let chroma = value / saturation;
		let hue_prime = hue / 60.0;
		let x = chroma * (1.0 - (hue_prime % 2.0 - 1.0).abs());

		let (mut red, mut green, mut blue) = (0.0, 0.0, 0.0);
		if hue_prime < 1.0 {
			red = chroma;
			green = x;
		} else if hue_prime < 2.0 {
			red = x;
			green = chroma;
		} else if hue_prime < 3.0 {
			green = chroma;
			blue = x;
		} else if hue_prime < 4.0 {
			green = x;
			blue = chroma;
		} else if hue_prime < 5.0 {
			red = x;
			blue = chroma;
		} else {
			red = chroma;
			blue = x;
		}

		let m = value - chroma;
		vec![red + m, green + m, blue + m]
	}

	pub fn from_hsl(channels: Channels) -> Channels {
		let [hue, saturation, lightness] = channels[..] else { panic!("Failed to unwrap channels!") };

		let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
		let hue_prime = hue / 60.0;
		let x = chroma * (1.0 - (hue_prime % 2.0 - 1.0).abs());

		let (mut red, mut green, mut blue) = (0.0, 0.0, 0.0);
		if hue_prime < 1.0 {
			red = chroma;
			green = x;
		} else if hue_prime < 2.0 {
			red = x;
			green = chroma;
		} else if hue_prime < 3.0 {
			green = chroma;
			blue = x;
		} else if hue_prime < 4.0 {
			green = x;
			blue = chroma;
		} else if hue_prime < 5.0 {
			red = x;
			blue = chroma;
		} else {
			red = chroma;
			blue = x;
		}

		let m = lightness - chroma / 2.0;
		vec![red + m, green + m, blue + m]
	}

	pub fn from_cmyk(channels: Channels) -> Channels {
		let [cyan, magenta, yellow, black] = channels[..] else { panic!("Failed to unwrap channels!") };
		let white = 1.0 - black;

		vec![
			(1.0 - cyan) * white,
			(1.0 - magenta) * white,
			(1.0 - yellow) * white
		]
	}

	pub fn from_xyz(channels: Channels) -> Channels {
		let [x, y, z] = channels[..] else { panic!("Failed to unwrap channels!") };
		let (red, green, blue) = (
			3.2406 * x - 1.5372 * y - 0.4986 * z,
			-0.9689 * x + 1.8758 * y + 0.0415 * z,
			0.0557 * x - 0.2040 * y + 1.0570 * z
		);

		// This conversion cause overflows
		vec![
			clamp_channel!(f64: red),
			clamp_channel!(f64: green),
			clamp_channel!(f64: blue)
		]
	}

	pub fn from_lab(channels: Channels) -> Channels {
		let [lightness, a, b] = channels[..] else { panic!("Failed to unwrap channels!") };
		let (x_ref, y_ref, z_ref) = (0.9642, 1.0, 0.8251);

		let yr = (lightness + 16.0) / 116.0;
		let xr = a / 500.0 + yr;
		let zr = yr - b / 200.0;
		let (xr3, yr3, zr3) = (xr.powi(3), yr.powi(3), zr.powi(3));

		let epsilon = 0.008856;
		from_xyz(vec![
			if xr3 > epsilon { xr3 } else { (116.0 * xr - 16.0) / 903.3 },
			if yr3 > epsilon { yr3 } else { (116.0 * yr - 16.0) / 903.3 },
			if zr3 > epsilon { zr3 } else { (116.0 * zr - 16.0) / 903.3 }
		])
	}

	pub fn from_lch(channels: Channels) -> Channels {
		let [luminance, chroma, hue] = channels[..] else { panic!("Failed to unwrap channels!") };

		from_lab(vec![
			luminance,
			chroma * hue.to_radians().cos(),
			chroma * hue.to_radians().sin()
		])
	}
}

#[test]
fn test() {
	let color = vec![0.2, 0.5, 0.7];
	println!("{:?}", from_rgb::to_lab(color))
}