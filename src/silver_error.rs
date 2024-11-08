use image::ImageError;
use serde::ser::StdError;
use std::array::TryFromSliceError;
use std::string::FromUtf8Error;
use std::{fmt, io};

/// Possible errors encountered when parsing, or etc.
#[derive(Debug)]
pub enum SilverError {
    ContentParseFailure(FromUtf8Error),
    InvalidHeader,
    ParseError(io::Error),
    InvalidMagic,
    InvalidBitmap,
    ImageError(ImageError),
}

impl From<io::Error> for SilverError {
    fn from(value: io::Error) -> Self {
        SilverError::ParseError(value)
    }
}

// Used for when parsing string contents.
impl From<FromUtf8Error> for SilverError {
    fn from(value: FromUtf8Error) -> Self {
        SilverError::ContentParseFailure(value)
    }
}

// Used for mismatches when reading magic.
impl From<TryFromSliceError> for SilverError {
    fn from(_: TryFromSliceError) -> Self {
        SilverError::InvalidMagic
    }
}

// Used for when converting between images and bitmaps.
impl From<ImageError> for SilverError {
    fn from(value: ImageError) -> Self {
        SilverError::ImageError(value)
    }
}

impl fmt::Display for SilverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContentParseFailure(e) => write!(f, "Failed to parse section content: {e}"),
            Self::InvalidMagic => write!(f, "Invalid magic detected!"),
            Self::ParseError(e) => write!(f, "Failed to parse file format: {e}"),
            Self::InvalidHeader => write!(f, "Invalid header for SilverDB file encountered!"),
            Self::InvalidBitmap => write!(f, "Invalid bitmap resource entry encountered!"),
            Self::ImageError(e) => write!(f, "Failed to convert image: {}", e),
        }
    }
}

impl StdError for SilverError {
    fn description(&self) -> &str {
        match self {
            Self::ContentParseFailure(_) => "Failed to parse section content.",
            Self::InvalidMagic => "Invalid magic detected!",
            Self::ParseError(_) => "Failed to parse file format.",
            Self::InvalidHeader => "Invalid header for SilverDB file encountered!",
            Self::InvalidBitmap => "Invalid bitmap resource entry encountered!",
            Self::ImageError(_) => "Failed to convert image.",
        }
    }
}
