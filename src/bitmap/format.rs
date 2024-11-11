use std::io::Cursor;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{little_helper::LittleHelper, SilverError};

/// Possible representations of bitmap data.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum RawBitmapType {
    GrayscaleTwo = 0x0002,
    GrayscaleFour = 0x0004,
    GrayscaleEight = 0x0008,
    Rgb565 = 0x0565,
    Argb4444 = 0x1444,
    Argb8888 = 0x1888,
    RgbEight = 0x0064,
    RgbSixteen = 0x0065,
}

impl TryFrom<u16> for RawBitmapType {
    type Error = SilverError;

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            x if x == RawBitmapType::GrayscaleTwo as u16 => Ok(RawBitmapType::GrayscaleTwo),
            x if x == RawBitmapType::GrayscaleFour as u16 => Ok(RawBitmapType::GrayscaleFour),
            x if x == RawBitmapType::GrayscaleEight as u16 => Ok(RawBitmapType::GrayscaleEight),
            x if x == RawBitmapType::Rgb565 as u16 => Ok(RawBitmapType::Rgb565),
            x if x == RawBitmapType::Argb4444 as u16 => Ok(RawBitmapType::Argb4444),
            x if x == RawBitmapType::Argb8888 as u16 => Ok(RawBitmapType::Argb8888),
            x if x == RawBitmapType::RgbEight as u16 => Ok(RawBitmapType::RgbEight),
            x if x == RawBitmapType::RgbSixteen as u16 => Ok(RawBitmapType::RgbSixteen),
            _ => Err(SilverError::UnknownBitmap),
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
    /// The color depth associated with this image.
    /// (This is correlated to the image's bitmap type.)
    pub color_depth: u16,
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
        let cursor = Cursor::new(raw_data.clone());
        let mut helper = LittleHelper(cursor);

        let mut representation = RawBitmapData {
            image_type: helper.read_u16_le()?.try_into()?,
            is_external: helper.read_u16_le()?,
            rendered_width: helper.read_u16_le()?,
            color_depth: helper.read_u16_le()?,
            padding_one: helper.read_u32_le()?,
            padding_two: helper.read_u32_le()?,
            width: helper.read_u32_le()?,
            height: helper.read_u32_le()?,
            // These will be filled in below.
            resource_id: 0,
            contents_length: 0,
            // We'll read once we can access the content length.
            contents: vec![],
        };

        // Some older firmware (e.g. for iPod classics) lacks the resource ID
        // field within the bitmap-specific resource header, and skip
        // directly to the bitmap `contents_length` field.
        //
        // Thankfully, we can logically infer what this value is intended to be:
        // On iPods which do have resource IDs present (e.g. iPod nanos),
        // we should expect to see a resource ID of 0xDAD0_0000 or greater.
        // Alternatively, we may see a resource ID of 0 for some `StBm` images.
        let next_u32 = helper.read_u32_le()?;
        if next_u32 >= 0x0DAD_0000 || next_u32 == 0 {
            // We'll fill in the resource ID, and read the actual content length.
            representation.resource_id = next_u32;
            representation.contents_length = helper.read_u32_le()?;
        } else {
            representation.contents_length = next_u32;
        }

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
