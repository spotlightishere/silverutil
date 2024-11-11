use std::io::Cursor;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{little_helper::LittleHelper, SilverError};

/// Possible representations of bitmap data.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum RawBitmapType {
    Grayscale = 0x0001,
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
            x if x == RawBitmapType::Grayscale as u16 => Ok(RawBitmapType::Grayscale),
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

/// A fictional type to help our marshaller/demarshaller
/// determine how we should interpret bitmap header info.
#[derive(Deserialize, Serialize, Debug)]
pub enum FirmwareType {
    /// A bitmap header containing a valid resource ID, typically on iPod nanos.
    Nano,
    /// A bitmap header lacking a valid resource ID, typically on LCD iPods.
    LCD,
    /// A bitmap header oriented for monochrome iPods.
    Monochrome,
}

#[derive(Deserialize, Serialize, Debug)]
/// A somewhat raw representation of bitmap data
/// within a resource entry's raw contents.
///
/// Due to a difference between implementations,
/// this is not a binary-equivalent structure.
pub struct RawBitmapData {
    /// The type of this bitmap image.
    pub image_type: RawBitmapType,
    /// This field's exact usage is unknown: it was observed to be one
    /// within external SilverImages on the N7G, and zero within internal bitmaps.
    // TODO(spotlightishere): Determine exact usage
    pub is_external: u16,
    /// The width this bitmap image will be rendered at.
    /// On monochrome devices, this is zero as it does not exist.
    pub rendered_width: u16,
    /// The color depth associated with this image.
    /// (This is correlated to the image's bitmap type.)
    pub color_depth: u16,
    /// The width of this bitmap image.
    pub width: u32,
    /// The height of this bitmap image.
    pub height: u32,
    /// The resource ID specified by this bitmap image.
    /// A resource ID is only available on Nano firmwares:
    /// this value will be set to zero.
    ///
    /// Please note that this value being zero does not indicate that
    /// the header is for a non-nano device: some nano firmwares
    /// have bitmap resources with a resource ID of zero.
    pub resource_id: u32,
    /// The length of contents, as provided by the header.
    /// The length of the actual read contents will be validated.
    pub contents_length: u32,
    /// The raw, binary contents of the bitmap image.
    pub contents: Vec<u8>,
    /// A fictional type to allow us to differentiate between
    /// header types whilst marshalling.
    pub header_format: FirmwareType,
}

impl RawBitmapData {
    /// Initializes a new [`RawBitmapData`] with dummy data.
    fn new() -> Self {
        RawBitmapData {
            image_type: RawBitmapType::Rgb565,
            is_external: 0,
            rendered_width: 0,
            color_depth: 0,
            width: 0,
            height: 0,
            resource_id: 0,
            contents_length: 0,
            contents: vec![],
            header_format: FirmwareType::Nano,
        }
    }
}

impl RawBitmapData {
    /// Parses a resource entry's raw contents to our representation.
    pub fn parse(raw_data: Vec<u8>) -> Result<Self, SilverError> {
        // Read our entire internal representation.
        let resource_length = raw_data.len() as u32;
        let cursor = Cursor::new(raw_data.clone());
        let mut helper = LittleHelper(cursor);

        let mut representation = RawBitmapData::new();
        // Read our first byte.
        //
        // In some older firmware versions (e.g. iPod mini),
        // the first byte is 0x0, and our image type is specified later on.
        // In later firmwares, the first u16 is our image type.
        let first_u16 = helper.read_u16_le()?;
        if first_u16 == 0 {
            // As our first u16 was 0, we immediately know that
            // this firmware type is for older, monochrome iPods.
            representation.header_format = FirmwareType::Monochrome;

            // The next u16 and following u32 should also be zero.
            // (That is, the first 0x8 bytes are all zero.)
            let padding_u16 = helper.read_u16_le()?;
            let padding_u32 = helper.read_u32_le()?;
            if padding_u16 != 0 && padding_u32 != 0 {
                panic!("padding found to be non-zero in iPod monochrome firmware!");
            }

            // We then have height and width.
            representation.height = helper.read_u32_le()?;
            representation.width = helper.read_u32_le()?;

            // TODO(spotlightishere): What is this?
            let unknown_value = helper.read_u16_le()?;
            println!("Unknown value: {}", unknown_value);

            // Our image type is in the following u16.
            let presumably_image_type = helper.read_u16_le()?;
            println!("Possibly image type: {presumably_image_type}");
            representation.image_type = presumably_image_type.try_into()?;

            // Followwing that, we again have padding.
            let padding_two = helper.read_u32_le()?;
            if padding_two != 0 {
                panic!("secondary padding found to be non-zero in monochrome iPod firmware!");
            }

            // Our content length follows, thus concluding our header.
            representation.contents_length = helper.read_u32_le()?;
        } else {
            // As this is a Nano or Color firmware,
            // our first byte is the image type.
            representation.image_type = first_u16.try_into()?;

            // We'll then read this format sequentially.
            // Our first u16 was our image type.
            // We then have flags of sorts,
            representation.is_external = helper.read_u16_le()?;
            representation.rendered_width = helper.read_u16_le()?;
            representation.color_depth = helper.read_u16_le()?;

            // We should have two bytes of padding here.
            // Per known firmware, these values are 0x0.
            //
            // Though you should generally not panic in libraries,
            // we're going to do such here as this likely means that
            // the parsed bitmap image type is not one we're aware of.
            let padding_one = helper.read_u32_le()?;
            let padding_two = helper.read_u32_le()?;
            if padding_one != 0 && padding_two != 0 {
                panic!("padding found to be non-zero in iPod nano/LCD firmware!");
            }

            representation.height = helper.read_u32_le()?;
            representation.width = helper.read_u32_le()?;

            // Some older firmware (e.g. for iPod classics) lacks the resource ID
            // field within the bitmap-specific resource header, and skip
            // directly to the bitmap `contents_length` field.
            //
            // Thankfully, we can logically infer what this value is intended to be:
            // On iPods which do have resource IDs present (e.g. iPod nanos),
            // we should expect to see a resource ID of 0xDAD0_0000 or greater.
            // Alternatively, we may see a resource ID of 0 for some `StBm` images on Nanos.
            let next_u32 = helper.read_u32_le()?;
            if next_u32 >= 0x0DAD_0000 || next_u32 == 0 {
                // We'll fill in the resource ID, and read the actual content length.
                representation.resource_id = next_u32;
                representation.contents_length = helper.read_u32_le()?;
                representation.header_format = FirmwareType::Nano;
            } else {
                representation.contents_length = next_u32;
                representation.header_format = FirmwareType::LCD;
            }
        }

        // Ensure that we have exactly enough data available to read the content's length.
        let remaining_data = resource_length - helper.pos_as_u32();
        if representation.contents_length != remaining_data {
            return Err(SilverError::InvalidBitmap);
        }

        println!("parsed: {:?}", representation);

        representation.contents = helper.read_length(representation.contents_length)?;
        Ok(representation)
    }

    // Reduces our representation to a resource entry's raw contents.
    pub fn _reduce(self) -> Result<Vec<u8>, SilverError> {
        todo!()
    }
}
