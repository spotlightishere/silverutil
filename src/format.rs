use std::fmt;

use binrw::binrw;

#[binrw]
#[brw(little)]
pub struct SilverDB {
    pub header: SilverDBHeader,
    #[br(count = header.section_count)]
    pub sections: Vec<SectionHeader>,
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
    pub file_count: u32,
    // Possibly related to flags?
    pub unknown_value: u32,
    // Offset to the section, relative to the start of the file (0x0).
    pub file_entry_offset: u32,
}

#[binrw]
pub struct FileEntry {
    pub id: u32,
    pub offset: u32,
    pub size: u32,
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
