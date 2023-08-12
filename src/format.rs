use binrw::{args, binrw, io::SeekFrom};

#[binrw]
#[brw(little)]
pub struct SilverDBFormat {
    pub header: SilverDBHeader,

    // When reading, we need to pass on the header length to sections
    // so that resource content location can be determined.
    #[br(args { inner: args! { header_length: header.header_length } }, count = header.section_count)]
    pub sections: Vec<SectionHeader>,
}

#[binrw]
pub struct SilverDBHeader {
    // 0x03 across 5th, 6th, and 7th generation iPod nanos.
    pub version: u32,
    // The length consumed by header content.
    // Resource data begins immediately after all header values.
    pub header_length: u32,
    // The amount of sections this database contains.
    pub section_count: u32,
}

/// Simply a u32, but we read it as an array to ensure endianness.
pub type SectionMagic = [u8; 4];

#[binrw]
#[br(import {
    header_length: u32
})]
pub struct SectionHeader {
    // The magic identifying this section (i.e. 'Str ', 'BMap', 'LDTm', etc.)
    pub magic: SectionMagic,
    // The amount of resources contained within this section.
    pub resource_count: u32,
    // Possibly flags for this section?
    pub unknown_value: u32,
    // Offset to array of resource entries, relative to the start of the file (0x0).
    pub resource_offset: u32,

    // Similar to SilverDBFormat, we need to pass on the header length to
    // sections so that resource content location can be determined.
    #[br(args { inner: args! { header_length } }, count = resource_count, seek_before = SeekFrom::Start(resource_offset.into()), restore_position)]
    pub resources: Vec<ResourceMetadata>,
}

#[binrw]
#[br(import {
    header_length: u32
})]
pub struct ResourceMetadata {
    // The ID is how this resource is referenced. For example, 0x0dad06d8.
    pub id: u32,
    // The offset to where this resource's data is located.
    // This is relative to where data begins (i.e. after header, section header, and resource entries.)
    pub data_offset: u32,
    // The length of this resource.
    pub data_size: u32,

    // Again, our data is relative to where the header ends.
    // We know this location via the passed `header_length` argument.
    #[br(seek_before = SeekFrom::Start((header_length + data_offset).into()), count=data_size, restore_position)]
    pub contents: Vec<u8>,
}
