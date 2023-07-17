use std::fmt;

use binrw::{binrw, io::SeekFrom, PosValue};

#[binrw]
#[brw(little)]
pub struct SilverDBFormat {
    pub header: SilverDBHeader,
    #[br(count = header.section_count)]
    pub sections: Vec<SectionHeader>,

    // TODO(spotlightishere): This should be removed once proper offset
    // determination via binrw itself is figured out.
    // TODO(spotlightishere): Please remove the internal offset hack...
    #[br(seek_before = SeekFrom::Start(sections.last().expect("should have last section").resources.last().expect("should have last resource").internal_offset.pos), parse_with = binrw::until_eof)]
    #[bw(ignore)]
    pub remaining_data: Vec<u8>,
}

#[binrw]
pub struct SilverDBHeader {
    // 0x03 across 5th, 6th, and 7th generation iPod nanos.
    pub version: u32,
    // Unknown - possibly length related?
    pub unknown_value: u32,
    // The amount of sections this database contains.
    pub section_count: u32,
}

#[binrw]
pub struct SectionHeader {
    // The magic identifying this section (i.e. 'Str ', 'BMap', 'LDTm', etc.)
    pub magic: FourCC,
    // The amount of resources contained within this section.
    pub resource_count: u32,
    // Possibly flags for this section?
    pub unknown_value: u32,
    // Offset to array of resource entries, relative to the start of the file (0x0).
    pub resource_offset: u32,

    #[br(count = resource_count, seek_before = u32_seek_offset(&resource_offset), restore_position)]
    pub resources: Vec<ResourceMetadata>,
}

/// Helper to assist passing with `SeekFrom` because it does not
/// permit dereferencing otherwise.
fn u32_seek_offset(offest: &u32) -> SeekFrom {
    SeekFrom::Start((*offest).into())
}

#[binrw]
pub struct ResourceMetadata {
    // The ID is how this resource is referenced. For example, 0x0dad06d8.
    pub id: u32,
    // The offset to where this resource's data is located.
    // This is relative to where data begins (i.e. after header, section header, and resource entries.)
    pub data_offset: u32,
    // The length of this resource.
    pub data_size: u32,

    // TODO(spotlightishere): Remove
    // Cheap hack to determine final offset of data
    #[bw(ignore)]
    internal_offset: PosValue<()>,
}

/// Since we cannot implement on type aliases, this struct (containing a single u32)
/// assists to help us display this magic as both its ASCII and hexadecimal representations.
#[binrw]
pub struct FourCC {
    pub magic: u32,
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // We parsed in little endian, so now we handle this as
        // big endian in order to have it be readable.
        let magic_bytes = self.magic.to_be_bytes();
        let magic_string =
            String::from_utf8(magic_bytes.to_vec()).unwrap_or("invalid ASCII magic".to_string());
        write!(f, "{:?} (0x{:08x})", magic_string, self.magic)?;

        Ok(())
    }
}
