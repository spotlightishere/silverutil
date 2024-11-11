use std::io::Cursor;

use crate::{
    bitmap::format::{FirmwareType, RawBitmapData, RawBitmapType},
    SilverError,
};
use image::{GrayImage, RgbImage, RgbaImage};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
/// A higher-level representation of bitmap imagery.
/// It holds minimal information to lower to an internal representation,
/// alongside re-encoding the bitmap image to a PNG.
///
/// It's assumed that `contents` is a PNG.
pub struct BitmapImage {
    /// The width of our bitmap data.
    pub width: u32,
    /// The height of our bitmap data.
    pub height: u32,
    /// The height this bitmap image is rendered at.
    pub rendered_width: u16,
    /// The format this bitmap image is encoded in.
    pub format_type: RawBitmapType,
    /// The resource ID this bitmap data is associated with.
    pub resource_id: u32,
    /// A PNG-encoded version of our bitmap image.
    pub contents: Vec<u8>,
}

impl BitmapImage {
    /// Parses a resource entry's raw contents to our representation.
    pub fn parse(raw_data: Vec<u8>) -> Result<Option<Self>, SilverError> {
        // Some bitmap images have no data. It's unclear why this is.
        if raw_data.is_empty() {
            return Ok(None);
        }

        println!("Raw data: {}", hex::encode(raw_data.clone()));
        let raw_format = RawBitmapData::parse(raw_data)?;

        // TODO(spotlightishere): Remove
        let resource_id = raw_format.resource_id;
        println!("{} is {:?}", resource_id, raw_format.image_type);
        println!("\tColor depth: {:?}", raw_format.color_depth);
        println!(
            "\tDimensions: {}x{} (rendered at width {})",
            raw_format.width, raw_format.height, raw_format.rendered_width
        );

        // Additionally, some bitmap images have dimensions, but lack any substance.
        // TODO(spotlightishere): Is this correct?
        if raw_format.contents_length == 0 || raw_format.contents.is_empty() {
            return Ok(None);
        }

        // On iPod nanos or iPods equipped with LCDs,
        // we need to respect the rendered_width field.
        //
        // TODO(spotlightishere): Is this really how the rendered_width field is used,
        // or are there fields indicating orientation (landscape/portrait)?
        let height = raw_format.height;
        let width = match raw_format.header_format {
            FirmwareType::Monochrome => raw_format.width,
            FirmwareType::Nano | FirmwareType::LCD => match raw_format.image_type {
                RawBitmapType::GrayscaleTwo => raw_format.rendered_width as u32 * 4,
                RawBitmapType::GrayscaleFour => raw_format.rendered_width as u32 * 2,
                RawBitmapType::GrayscaleEight => raw_format.rendered_width as u32,
                _ => raw_format.width,
            },
        };

        // Now, convert our bitmap data to a PNG representation.
        let mut png_writer = Cursor::new(Vec::new());
        match raw_format.image_type {
            RawBitmapType::Grayscale => {
                // We have eight pixels in every byte.
                let gray_contents: Vec<u8> = raw_format
                    .contents
                    .into_iter()
                    .flat_map(|pixel| {
                        let one: u8 = (pixel & 0x1) * 255;
                        let two: u8 = ((pixel >> 1) & 0x1) * 255;
                        let three: u8 = ((pixel >> 2) & 0x1) * 255;
                        let four: u8 = ((pixel >> 3) & 0x1) * 255;
                        let five: u8 = ((pixel >> 4) & 0x1) * 255;
                        let six: u8 = ((pixel >> 5) & 0x1) * 255;
                        let seven: u8 = ((pixel >> 6) & 0x1) * 255;
                        let eight: u8 = ((pixel >> 7) & 0x1) * 255;

                        [eight, seven, six, five, four, three, two, one]
                    })
                    .collect();

                let gray_image = GrayImage::from_raw(width, height, gray_contents)
                    .expect("should be able to create grayscale image");
                gray_image.write_to(&mut png_writer, image::ImageFormat::Png)?;
            }
            RawBitmapType::GrayscaleTwo => {
                // We have four pixels in every byte.
                let gray_contents: Vec<u8> = raw_format
                    .contents
                    .into_iter()
                    .flat_map(|pixel| {
                        let one: u8 = (pixel & 0x3) * 32;
                        let two: u8 = ((pixel >> 2) & 0x3) * 32;
                        let three: u8 = ((pixel >> 4) & 0x3) * 32;
                        let four: u8 = ((pixel >> 6) & 0x3) * 32;

                        [four, three, two, one]
                    })
                    .collect();

                let gray_image = GrayImage::from_raw(width, height, gray_contents)
                    .expect("should be able to create grayscale image");
                gray_image.write_to(&mut png_writer, image::ImageFormat::Png)?;
            }
            RawBitmapType::GrayscaleFour => {
                // We have two pixels in every byte.
                let gray_contents: Vec<u8> = raw_format
                    .contents
                    .into_iter()
                    .flat_map(|pixel| {
                        let lower: u8 = ((pixel >> 4) & 0xf) * 16;
                        let upper: u8 = (pixel & 0xf) * 16;

                        [lower, upper]
                    })
                    .collect();

                let gray_image = GrayImage::from_raw(width, height, gray_contents)
                    .expect("should be able to create grayscale image");
                gray_image.write_to(&mut png_writer, image::ImageFormat::Png)?;
            }
            RawBitmapType::GrayscaleEight => {
                let gray_image = GrayImage::from_raw(width, height, raw_format.contents)
                    .expect("should be able to create grayscale image");
                gray_image.write_to(&mut png_writer, image::ImageFormat::Png)?;
            }
            RawBitmapType::Rgb565 => {
                // Parse our raw RGB565 contents by chunking every two bytes.
                // We can then manipulate each u16.
                let rgb_contents: Vec<u8> = raw_format
                    .contents
                    .chunks_exact(2)
                    .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                    .flat_map(|pixel| {
                        let r = ((pixel >> 11 & 0b11111) * (255 / 0b11111)) as u8;
                        let g = ((pixel >> 5 & 0b111111) * (255 / 0b111111)) as u8;
                        let b = ((pixel & 0b11111) * (255 / 0b11111)) as u8;

                        [r, g, b]
                    })
                    .collect();

                let rgb_image = RgbImage::from_raw(width, height, rgb_contents.clone())
                    .expect("should be able to create an RGB image");

                rgb_image.write_to(&mut png_writer, image::ImageFormat::Png)?;
            }
            RawBitmapType::Argb4444 => {
                let rgba_contents: Vec<u8> = raw_format
                    .contents
                    .chunks_exact(2)
                    .flat_map(|pixels| {
                        // This is little-endian, so ARGB is actually the reverse (BGRA).
                        let b: u8 = (pixels[0] & 0xf) * 16;
                        let g: u8 = ((pixels[0] >> 4) & 0xf) * 16;
                        let r: u8 = (pixels[1] & 0xf) * 16;
                        let a: u8 = ((pixels[1] >> 4) & 0xf) * 16;

                        [r, g, b, a]
                    })
                    .collect();

                let rgba_image = RgbaImage::from_raw(width, height, rgba_contents.clone())
                    .expect("should be able to create an RGBA image");

                rgba_image.write_to(&mut png_writer, image::ImageFormat::Png)?;
            }
            RawBitmapType::Argb8888 => {
                let rgba_contents: Vec<u8> = raw_format
                    .contents
                    .chunks_exact(4)
                    .flat_map(|pixels| {
                        // This is little-endian, so ARGB is actually the reverse (BGRA).
                        let b = pixels[0];
                        let g = pixels[1];
                        let r = pixels[2];
                        let a = pixels[3];

                        [r, g, b, a]
                    })
                    .collect();

                let rgba_image = RgbaImage::from_raw(width, height, rgba_contents.clone())
                    .expect("should be able to create an RGBA image");

                rgba_image.write_to(&mut png_writer, image::ImageFormat::Png)?;
            }
            RawBitmapType::RgbEight => {
                // Obtain our palette and raw, indexed contents.
                let (palette, indexed_contents) = separate_palette(raw_format.contents);

                // Iterate through each chunk and resolve RGBA colors from our palette.
                let rgba_contents = indexed_contents
                    .iter()
                    .flat_map(|index| {
                        // Our palette is ARGB.
                        let (r, g, b, a) = palette[*index as usize];
                        [r, g, b, a]
                    })
                    .collect();

                // Finally, create our image.
                let rgba_image = RgbaImage::from_raw(width, height, rgba_contents)
                    .expect("should be able to create an RGBA image");

                rgba_image.write_to(&mut png_writer, image::ImageFormat::Png)?;
            }
            RawBitmapType::RgbSixteen => {
                // Obtain our palette and raw, indexed contents.
                let (palette, indexed_contents) = separate_palette(raw_format.contents);

                // Iterate through each chunk and resolve RGBA colors from our palette.
                // As we're 16-bit, map our index from two u8 to one u16.
                let rgba_contents = indexed_contents
                    .chunks_exact(2)
                    .map(|x| u16::from_le_bytes([x[0], x[1]]))
                    .flat_map(|index| {
                        // Our palette is ARGB.
                        let (r, g, b, a) = palette[index as usize];
                        [r, g, b, a]
                    })
                    .collect();

                // Finally, create our image.
                let rgba_image = RgbaImage::from_raw(width, height, rgba_contents)
                    .expect("should be able to create an RGBA image");

                rgba_image.write_to(&mut png_writer, image::ImageFormat::Png)?;
            }
        }

        let result = BitmapImage {
            width: raw_format.width,
            height: raw_format.width,
            format_type: raw_format.image_type,
            rendered_width: raw_format.rendered_width,
            resource_id: raw_format.resource_id,
            contents: png_writer.into_inner(),
        };
        Ok(Some(result))
    }

    // Reduces our representation to a resource entry's raw contents.
    pub fn reduce(self) -> Result<Vec<u8>, SilverError> {
        todo!("reducing is not yet supported")
    }
}

/// An ARGB pixel.
type ArgbPixel = (u8, u8, u8, u8);

/// Reads the four first bytes of our raw contents
/// in order to separate the palette from our contents.
/// It then returns the palette as a tuple of four u8s,
/// and the raw contents as a simple Vec<u8>.
fn separate_palette(raw_contents: Vec<u8>) -> (Vec<ArgbPixel>, Vec<u8>) {
    let palette_length = u32::from_le_bytes([
        raw_contents[0],
        raw_contents[1],
        raw_contents[2],
        raw_contents[3],
    ]);

    // The palette begins immediately after our length, a u32.
    // It's an array of RGBA8888, so we operate over clusters of four bytes.
    // However, because this is little endian, we read it as the inverse, ABGR8888.
    let palette_start = 4;
    let palette_end = palette_start + (palette_length as usize * 4);
    let palette = raw_contents[palette_start..palette_end]
        .chunks_exact(4)
        .flat_map(|pixels| {
            let b = pixels[0];
            let g = pixels[1];
            let r = pixels[2];
            let a = pixels[3];

            [(r, g, b, a)]
        })
        .collect();

    // Finally, separate our raw, indexed contents.
    let indexed_contents = &raw_contents[palette_end..];
    (palette, indexed_contents.to_vec())
}
