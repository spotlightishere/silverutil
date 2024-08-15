use std::io::Cursor;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{little_helper::LittleHelper, SilverError};

/// Possible representations of bitmap data.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum RawBitmapType {
    GrayscaleFour = 0x0004,
    GrayscaleEight = 0x0008,
    Rgb565 = 0x0565,
    Argb8888 = 0x1888,
    RgbEight = 0x0064,
    RgbSixteen = 0x0065,
}

impl TryFrom<u16> for RawBitmapType {
    type Error = SilverError;

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            x if x == RawBitmapType::GrayscaleFour as u16 => Ok(RawBitmapType::GrayscaleFour),
            x if x == RawBitmapType::GrayscaleEight as u16 => Ok(RawBitmapType::GrayscaleEight),
            x if x == RawBitmapType::Rgb565 as u16 => Ok(RawBitmapType::Rgb565),
            x if x == RawBitmapType::Argb8888 as u16 => Ok(RawBitmapType::Argb8888),
            x if x == RawBitmapType::RgbEight as u16 => Ok(RawBitmapType::RgbEight),
            x if x == RawBitmapType::RgbSixteen as u16 => Ok(RawBitmapType::RgbSixteen),
            _ => Err(SilverError::InvalidBitmap),
        }
    }
}

#[derive(Deserialize, Serialize)]
/// The raw representation of bitmap data within a resource entry's raw contents.
pub struct RawBitmapData {
    /// The type of this bitmap image.
    pub image_type: RawBitmapType,
    /// This field's exact usage is unknown: it was observed to be one
    /// within external SilverImages, and zero within internal bitmaps.
    // TODO(spotlightishere): Determine exact usage
    pub is_external: u16,
    /// The width this bitmap image will be rendered at.
    pub rendered_width: u16,
    /// Possibly flags associated with this image.
    pub flags: u16,
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
            image_type: helper.read_u16_le()?.try_into()?,
            is_external: helper.read_u16_le()?,
            rendered_width: helper.read_u16_le()?,
            flags: helper.read_u16_le()?,
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
