use std::io::Cursor;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{little_helper::LittleHelper, SilverError};

/// Possible representations of bitmap data.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum RawBitmapType {
    DepthFour = 0x04,
    DepthEight = 0x08,
    NoClue = 0x64,
    Rgb565 = 0x65,
    EightyEight = 0x88,
}

impl TryFrom<u8> for RawBitmapType {
    type Error = SilverError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == RawBitmapType::DepthFour as u8 => Ok(RawBitmapType::DepthFour),
            x if x == RawBitmapType::DepthEight as u8 => Ok(RawBitmapType::DepthEight),
            x if x == RawBitmapType::NoClue as u8 => Ok(RawBitmapType::NoClue),
            x if x == RawBitmapType::Rgb565 as u8 => Ok(RawBitmapType::Rgb565),
            x if x == RawBitmapType::EightyEight as u8 => Ok(RawBitmapType::EightyEight),
            _ => Err(SilverError::InvalidBitmap),
        }
    }
}

#[derive(Deserialize, Serialize)]
/// The raw representation of bitmap data within a resource entry's raw contents.
pub struct RawBitmapData {
    /// The type of this bitmap image.
    pub image_type: RawBitmapType,
    /// Possibly the interpretation.
    // TODO(spotlightishere): Determine.
    pub image_mode: u8,
    /// Possibly the depth of this image.
    pub maybe_depth: u16,
    /// Unknown. Often 16.
    pub unknown_value: u16,
    /// Unknown. Often zero.
    pub unknown_again: u16,
    pub padding_one: u32,
    pub padding_two: u32,
    pub width: u32,
    pub height: u32,
    pub resource_id: u32,
    pub contents_length: u32,
    pub contents: Vec<u8>,
}

impl RawBitmapData {
    /// Parses a resource entry's raw contents to our representation.
    pub fn parse(raw_data: Vec<u8>) -> Result<Self, SilverError> {
        // Read the internal representation.
        let resource_length = raw_data.len() as u32;
        let cursor = Cursor::new(raw_data);
        let mut helper = LittleHelper(cursor);

        let mut representation = RawBitmapData {
            image_type: helper.read_u8()?.try_into()?,
            image_mode: helper.read_u8()?,
            maybe_depth: helper.read_u16_le()?,
            unknown_again: helper.read_u16_le()?,
            unknown_value: helper.read_u16_le()?,
            padding_one: helper.read_u32_le()?,
            padding_two: helper.read_u32_le()?,
            width: helper.read_u32_le()?,
            height: helper.read_u32_le()?,
            resource_id: helper.read_u32_le()?,
            contents_length: helper.read_u32_le()?,
            // We'll read once we can access the content length.
            contents: vec![],
        };

        // Ensure that we have exactly enough data available to read the content's length.
        let remaining_data = resource_length - helper.pos_as_u32();
        if representation.contents_length != remaining_data {
            return Err(SilverError::InvalidBitmap);
        }

        representation.contents = helper.read_length(representation.contents_length)?;
        Ok(representation)
    }

    // Reduces our representation to a resource entry's raw contents.
    pub fn _reduce(self) -> Result<Vec<u8>, SilverError> {
        todo!()
    }
}
