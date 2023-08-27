use serde::ser::StdError;
use std::array::TryFromSliceError;
use std::fmt;
use std::string::FromUtf8Error;

/// Possible errors encountered when parsing, or etc.
#[derive(Debug)]
pub enum SilverError {
    ContentParseFailure(FromUtf8Error),
    InvalidVersion,
    ParseError(binrw::Error),
    InvalidMagic,
}

impl From<binrw::Error> for SilverError {
    fn from(value: binrw::Error) -> Self {
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

impl fmt::Display for SilverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContentParseFailure(e) => write!(f, "Failed to parse section content: {e}"),
            Self::InvalidMagic => write!(f, "Invalid magic detected!"),
            Self::ParseError(e) => write!(f, "Failed to parse file format: {e}"),
            Self::InvalidVersion => write!(f, "Invalid version of SilverDB file encountered!"),
        }
    }
}

impl StdError for SilverError {
    fn description(&self) -> &str {
        match self {
            Self::ContentParseFailure(_) => "Failed to parse section content.",
            Self::InvalidMagic => "Invalid magic detected!",
            Self::ParseError(_) => "Failed to parse file format.",
            Self::InvalidVersion => "Invalid version of SilverDB file encountered!",
        }
    }
}
