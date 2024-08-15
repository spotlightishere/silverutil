use std::io::Cursor;

use crate::{
    bitmap::format::{RawBitmapData, RawBitmapType},
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
    /// The format this bitmap image is encoded in.
    pub format_type: RawBitmapType,
    /// The mode our bitmap data is encoded in.
    pub mode: u8,
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

        let raw_format = RawBitmapData::parse(raw_data.clone())?;
        let resource_id = raw_format.resource_id;

        // TODO(spotlightishere): Remove
        println!("=== Information ===");
        println!("\tResource ID: {}", resource_id);
        println!("\tResource Type: {:?}", raw_format.image_type);
        println!("\tResource Mode: {:?}", raw_format.image_mode);
        println!("\tWidth: {}", raw_format.width);
        println!("\tHeight: {}", raw_format.height);
        println!("\tClaimed size: {}", raw_format.contents_length);
        println!("\tBuffer size: {}", raw_format.contents.len());

        // Additionally, some bitmap images have dimensions, but lack any substance.
        // TODO(spotlightishere): Is this correct?
        if raw_format.contents_length == 0 || raw_format.contents.is_empty() {
            return Ok(None);
        }

        // TODO(spotlightishere): Better handle non-grayscale images
        if raw_format.width * raw_format.height > raw_format.contents.len() as u32 {
            return Ok(None);
        }

        // Now, convert our bitmap data to a PNG representation.
        let mut png_writer = Cursor::new(Vec::new());
        match raw_format.image_type {
            RawBitmapType::Rgb565 => {
                // Try and chip off what's presumably the palette.
                // TODO(spotlightishere): We likely should use this palette.
                let mut trimmed_contents = raw_format.contents.clone();
                let expected_size = if raw_format.image_mode == 0 {
                    raw_format.height * raw_format.width
                } else {
                    raw_format.height * raw_format.width * 2
                };

                if raw_format.contents_length > expected_size {
                    // We'll assume this palette is 4 bytes (or something).
                    // TODO(spotlightishere): Determine if this assumption can always be made
                    let palette_length = u32::from_le_bytes([
                        trimmed_contents[0],
                        trimmed_contents[1],
                        trimmed_contents[2],
                        trimmed_contents[3],
                    ]);
                    let removal_length = 4 + (palette_length * 4);
                    trimmed_contents.drain(0..removal_length as usize);
                }

                // TODO(spotlightishere): This is VERY wrong
                // Is this ARGB1555?
                if raw_format.image_mode == 0 {
                    let rgb_contents: Vec<u8> = trimmed_contents
                        .chunks_exact(2)
                        // Batch every pair of u8 as a u16.
                        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                        // Then, convert our u16 of RGB565 into three u8 of RGB.
                        .flat_map(|pixel| {
                            // TODO(spotlightishere): This alpha channel isn't correct.
                            // let a = ((pixel >> 12 & 0b1111) * (255 / 0b1111)) as u8;
                            let a = 0xFF;
                            let r = ((pixel >> 11 & 0b11111) * (255 / 0b11111)) as u8;
                            let g = ((pixel >> 6 & 0b11111) * (255 / 0b11111)) as u8;
                            let b = ((pixel >> 1 & 0b11111) * (255 / 0b11111)) as u8;

                            [r, g, b, a]
                        })
                        .collect();

                    let rgb_image = RgbaImage::from_raw(
                        raw_format.width,
                        raw_format.height,
                        rgb_contents.clone(),
                    )
                    .expect("should be able to create an RGBA image");

                    rgb_image.write_to(&mut png_writer, image::ImageFormat::Png)?;
                } else {
                    let rgb_contents: Vec<u8> = trimmed_contents
                        .chunks_exact(2)
                        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                        .flat_map(|pixel| {
                            let r = ((pixel >> 11 & 0b11111) * (255 / 0b11111)) as u8;
                            let g = ((pixel >> 5 & 0b111111) * (255 / 0b111111)) as u8;
                            let b = ((pixel & 0b11111) * (255 / 0b11111)) as u8;

                            [r, g, b]
                        })
                        .collect();

                    let rgb_image = RgbImage::from_raw(
                        raw_format.width,
                        raw_format.height,
                        rgb_contents.clone(),
                    )
                    .expect("should be able to create an RGB image");

                    rgb_image.write_to(&mut png_writer, image::ImageFormat::Png)?;
                }
            }
            // TODO(spotlightishere): This may be 16-bit RGB?
            // RawBitmapType::EightyEight => {
            //     let raw_contents: Vec<u16> = raw_format.contents
            //         .chunks_exact(2)
            //         .into_iter()
            //         .map(|x| u16::from_le_bytes([x[0], x[1]]))
            //         .collect();

            //     let gray_image = GrayAlpha16Image::from_raw(
            //         raw_format.width,
            //         raw_format.height,
            //         raw_contents
            //     )
            //     .expect("should be able to create grayscale alpha image");
            // }
            // TODO(spotlightishere): Flush this out in a better way
            _ => {
                let gray_image = GrayImage::from_raw(
                    raw_format.width,
                    raw_format.height,
                    raw_format.contents.clone(),
                )
                .expect("should be able to create grayscale image");

                gray_image.write_to(&mut png_writer, image::ImageFormat::Png)?;
            }
        }

        let result = BitmapImage {
            width: raw_format.width,
            height: raw_format.width,
            format_type: raw_format.image_type,
            mode: raw_format.image_mode,
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
