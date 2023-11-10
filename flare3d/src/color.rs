mod convertor;
pub mod colorspace;

/// Represents a color within a certain colorspace, stored as `f64` arrays.
///
/// Available colorspaces:
/// - **RGB**							- red, green, blue;
/// - **HSV**							- hue, saturation, value;
/// - **HSL**							- hue, saturation, lightness;
/// - **CMYK**							- cyan, magenta, yellow, black;
/// - **XYZ**							- x-axis, y-axis, z-axis;
/// - **Lab *(L\*a\*b\* or CIELAB)***	- Lightness, green-magenta, blue-yellow;
/// - **LCh *(HCL or CIELCh)***			- Luminance, Chroma, hue.
#[derive(Debug)]
pub enum Color {
    RGB([f64; 4]),
    HSV([f64; 4]),
    HSL([f64; 4]),
    CMYK([f64; 5]),
    XYZ([f64; 4]),
    Lab([f64; 4]),
    LCh([f64; 4]),
}

impl Color {
}

// Visualizations
impl Color {
    pub fn colorspace(&self) -> &'static str {
        match self {
            Color::RGB(_) => "rgb",
            Color::HSV(_) => "hsv",
            Color::HSL(_) => "hsl",
            Color::CMYK(_) => "cmyk",
            Color::XYZ(_) => "xyz",
            Color::Lab(_) => "L*a*b*",
            Color::LCh(_) => "LCh",
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.colorspace())
    }
}
