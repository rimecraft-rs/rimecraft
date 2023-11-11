use self::{colorspace::Colorspace, convertor::validate};

mod convertor;
pub mod colorspace;

pub type Channel = f64;
pub type Channels = [Channel; 4];

/// Represents a color within a certain colorspace, stored as `f64` arrays.
#[derive(Debug)]
pub struct Color {
	rgba: Channels,
	pub colorspace: Colorspace,
}

impl Color {
	pub fn new_rgb(colorspace: Colorspace, red: Channel, green: Channel, blue: Channel, alpha: Channel) -> Color {
		Color { rgba: validate([alpha, red, green, blue]), colorspace }
	}
}

impl Color {
    pub fn with_colorspace(&self, colorspace: Colorspace) -> Color {
        Color { colorspace, ..*self }
    }
}
