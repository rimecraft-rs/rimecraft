use rimecraft_fmt::Formatting;

/// An error type for the text module.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The given formatting does not contains a color.
    FormattingWithoutColor(Formatting),
    /// The color value is out of range.
    ColorValueOutOfRange(String),
    /// The color is invalid.
    InvalidColor(String),
    /// A formatting error.
    Formatting(rimecraft_fmt::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FormattingWithoutColor(formatting) => {
                write!(
                    f,
                    "the given formatting does not contains a color: {}",
                    formatting.raw_name(),
                )
            }
            Error::ColorValueOutOfRange(value) => {
                write!(f, "the color value is out of range: {value}")
            }
            Error::InvalidColor(value) => {
                write!(f, "the color is invalid: {value}")
            }
            Error::Formatting(err) => write!(f, "formatting error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<rimecraft_fmt::Error> for Error {
    #[inline]
    fn from(err: rimecraft_fmt::Error) -> Self {
        Error::Formatting(err)
    }
}
