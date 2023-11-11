use self::{colorspace::Colorspace, convertor::validate};

mod convertor;
mod convertible;
pub mod colorspace;
pub mod mix_mode;

pub type Channel = f64;
pub type Channels = Vec<Channel>;

/// Represents a color within a certain colorspace, stored as `f64` arrays.
#[derive(Debug)]
pub struct Color {
	channels: Channels,
	pub colorspace: Colorspace,
}

impl Color {
}

impl Color {
    pub fn with_colorspace(&self, colorspace: Colorspace) -> Color {
        Color { channels: self.channels.clone(), colorspace }
    }
}
