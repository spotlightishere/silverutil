use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{io, io::Cursor, io::Read, io::Write};

use crate::silver_error::SilverError;

/// Simply a u32, but we read it as an array to ensure endianness.
pub type SectionMagic = [u8; 4];

/// Helper to assist with reading and writing little-endian values.
struct LittleHelper(Cursor<Vec<u8>>);
impl LittleHelper {
    /// Creates a new LittleHelper with an empty Vec<u8>.
    fn new() -> Self {
        LittleHelper(Cursor::new(Vec::new()))
    }

    /// Returns the inner vector represented by this LittleHelper.
    fn contents(self) -> Vec<u8> {
        self.0.into_inner()
    }

    /// Seeks to the given u32.
    fn seek_to_u32(&mut self, offset: u32) {
        self.0.set_position(offset as u64);
    }

    /// Returns the current cursor's position as a u32.
    fn pos_as_u32(&mut self) -> u32 {
        self.0.position() as u32
    }

    /// Reads a little-endian u32.
    fn read_u32_le(&mut self) -> Result<u32, io::Error> {
        self.0.read_u32::<LittleEndian>()
    }

    /// Reads an arbitrary length of bytes.
    fn read_length(&mut self, length: u32) -> Result<Vec<u8>, io::Error> {
        let mut raw_data: Vec<u8> = vec![0; length as usize];
        self.0.read_exact(&mut raw_data)?;
        Ok(raw_data)
    }

    /// Reads our little-endian magic (that is, 4 bytes of u8).
    fn read_magic(&mut self) -> Result<SectionMagic, io::Error> {
        let mut raw_magic: SectionMagic = [0, 0, 0, 0];
        self.0.read_exact(&mut raw_magic)?;
        Ok(raw_magic)
    }

    /// Writes a little-endian u32.
    fn write_u32_le(&mut self, value: u32) -> Result<(), io::Error> {
        self.0.write_u32::<LittleEndian>(value)
    }

    /// Writes an arbitrary length of bytes.
    fn write_length(&mut self, raw_data: &Vec<u8>) -> Result<(), io::Error> {
        self.0.write_all(raw_data)
    }

    /// Writes our little-endian magic (that is, 4 bytes of u8).
    fn write_magic(&mut self, magic: SectionMagic) -> Result<(), io::Error> {
        self.0.write_all(&magic)
    }

    /// Writes padding to align this to 4 bytes.
    fn write_padding(&mut self, length: u32) -> Result<(), io::Error> {
        let padding_length = 4 - (length % 4);
        if padding_length == 4 {
            return Ok(());
        }

        let padding_data: Vec<u8> = vec![0; padding_length as usize];
        self.write_length(&padding_data)
    }
}

pub struct SilverDBFormat {
    pub header: SilverDBHeader,

    pub sections: Vec<SectionHeader>,
}

pub struct SilverDBHeader {
    /// 0x03 across 5th, 6th, and 7th generation iPod nanos.
    pub version: u32,

    /// The length consumed by header content.
    /// Resource data begins immediately after all header values.
    pub header_length: u32,

    /// The amount of sections this database contains.
    pub section_count: u32,
}

pub struct SectionHeader {
    /// The magic identifying this section (i.e. 'Str ', 'BMap', 'LDTm', etc.)
    pub magic: SectionMagic,

    /// The amount of resources contained within this section.
    pub resource_count: u32,

    /// Whether this section's IDs are sequential and start at 1. Necessary for certain resource types such as `StrT`.
    pub is_sequential: u32,

    /// Offset to array of resource entries, relative to the start of the file (0x0).
    pub resource_offset: u32,

    /// All available resources within this section.
    pub resources: Vec<ResourceMetadata>,
}

pub struct ResourceMetadata {
    /// The ID is how this resource is referenced. For example, 0x0dad06d8.
    pub id: u32,

    /// The offset to where this resource's data is located.
    /// This is relative to where data begins (i.e. after header, section header, and resource entries.)
    pub data_offset: u32,

    /// The length of this resource.
    pub data_size: u32,

    /// The raw, binary contents of this resource.
    pub contents: Vec<u8>,
}

impl SilverDBFormat {
    /// Reads a SilverDB-format file, returning a representation of its contents.
    pub fn read(raw_reader: Cursor<Vec<u8>>) -> Result<Self, SilverError> {
        let mut reader = LittleHelper(raw_reader);

        // First, read the header.
        let db_header = SilverDBHeader {
            version: reader.read_u32_le()?,
            header_length: reader.read_u32_le()?,
            section_count: reader.read_u32_le()?,
        };

        // Next, read all section metadata. Their metadata immediately follows the header.
        let mut db_sections: Vec<SectionHeader> = Vec::new();
        for _ in 0..db_header.section_count {
            let current_section = SectionHeader {
                magic: reader.read_magic()?,
                resource_count: reader.read_u32_le()?,
                is_sequential: reader.read_u32_le()?,
                resource_offset: reader.read_u32_le()?,
                // This will be backfilled once we finish reading all section metadata.
                resources: Vec::new(),
            };

            db_sections.push(current_section);
        }

        // Beyond section metadata, we have all per-resource metadata.
        // In practice, all resource metadata follows the order of their section.
        // However, we'll honor the file format and resolve the offset specified by section metadata.
        for current_section in db_sections.iter_mut() {
            reader.seek_to_u32(current_section.resource_offset);
            let mut section_resources: Vec<ResourceMetadata> = Vec::new();

            // Iterate through all resources in this section.
            for _ in 0..current_section.resource_count {
                let current_resource = ResourceMetadata {
                    id: reader.read_u32_le()?,
                    data_offset: reader.read_u32_le()?,
                    data_size: reader.read_u32_le()?,
                    // This will be backfilled once all resources are read.
                    contents: Vec::new(),
                };
                section_resources.push(current_resource);
            }

            current_section.resources = section_resources;
        }

        // Similarly, following resource metadata, we have our raw resource data.
        // Although section metadata's offsets to resource metadata are relative to the
        // start of the file, resource offsets are relative to the end of the file header.
        // (This is, the header length specified within the intiial header metadata.)
        //
        // We'll similarly honor the file format and jump around.
        for current_section in db_sections.iter_mut() {
            for current_resource in current_section.resources.iter_mut() {
                // We must seek relative to the end of the header.
                reader.seek_to_u32(db_header.header_length + current_resource.data_offset);
                let raw_contents = reader.read_length(current_resource.data_size)?;
                current_resource.contents = raw_contents;
            }
        }

        Ok(SilverDBFormat {
            header: db_header,
            sections: db_sections,
        })
    }

    /// Writes a representation of SilverDB contents to its binary format.
    pub fn write(&self) -> Result<Vec<u8>, SilverError> {
        // We'll have three writers: for our header/section metadata contents,
        // for our resource metadata, and for the raw resource data itself.
        let mut header_writer = LittleHelper::new();
        let mut resource_metadata_writer = LittleHelper::new();
        let mut raw_data_writer = LittleHelper::new();

        // Resource metadata will begin at the format header and the section metadatas.
        // The format header is 12 bytes in length, and section metadata is 16 (times section count).
        let resource_metadata_offset = 12 + (16 * self.header.section_count);

        // First, write out our database header.
        header_writer.write_u32_le(self.header.version)?;
        // This is SilverDBHeader's `header_length` - we'll go back and update it.
        header_writer.write_u32_le(0)?;
        header_writer.write_u32_le(self.header.section_count)?;

        // Next, start iterating through all sections.
        for current_section in self.sections.iter() {
            // First, obtain the current resource metadata offset for this section.
            // We'll use this to write the resource metadata offset within section metadata.
            let current_resource_meta_offset = resource_metadata_writer.pos_as_u32();

            // Next, write all resources and their subsequent resource metadatas.
            for current_resource in current_section.resources.iter() {
                // We'll use the current resource offset when writing its resourcce metadata.
                let current_raw_data_offset = raw_data_writer.pos_as_u32();
                raw_data_writer.write_length(&current_resource.contents)?;
                // All data is padded to 4 bytes, so we need to write padding.
                raw_data_writer.write_padding(current_resource.contents.len() as u32)?;

                // Now, we can write this resource's metadata.
                resource_metadata_writer.write_u32_le(current_resource.id)?;
                // This is ResourceMetadata's `data_offset`, which we marked previously.
                resource_metadata_writer.write_u32_le(current_raw_data_offset)?;
                resource_metadata_writer.write_u32_le(current_resource.data_size)?;
            }

            // Lastly, write this section's metadata.
            header_writer.write_magic(current_section.magic)?;
            header_writer.write_u32_le(current_section.resource_count)?;
            header_writer.write_u32_le(current_section.is_sequential)?;
            // This is SectionHeader's `resource_offset`, which was marked at the start of this loop.
            // We must adjust it to be past the format header + section headers (`resource_metadata_offset`).
            header_writer.write_u32_le(resource_metadata_offset + current_resource_meta_offset)?;

            // For an unknown reason, we must also pad the raw data for every section.
            // TODO(spotlightishere): It appears some sections require more than 4 bytes of alignment. Why?
            let current_raw_data_offset = raw_data_writer.pos_as_u32();
            raw_data_writer.write_padding(current_raw_data_offset)?;
        }

        // Before finalizing, we need to update the header to account for proper header length.
        let raw_resource_metadata = resource_metadata_writer.contents();
        let resource_metadata_length = raw_resource_metadata.len() as u32;
        // The total header length is the raw header/ plus raw resource metadata sizes.
        let total_header_length = header_writer.pos_as_u32() + resource_metadata_length;
        header_writer.seek_to_u32(4);
        header_writer.write_u32_le(total_header_length)?;

        // Combine all three writers, and we're done!
        let raw_header = header_writer.contents();
        let raw_data = raw_data_writer.contents();

        let raw_contents = [raw_header, raw_resource_metadata, raw_data].concat();
        Ok(raw_contents)
    }
}
