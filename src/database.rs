use std::{fs::File, io};

use binrw::{BinRead, Error};

use crate::format::SilverDBFormat;

/// A high-level representation of a SilverDB file.
pub struct SilverDB {
    /// Available sections within this database.
    pub sections: Vec<SilverSection>,
}

/// A high-level representation of section contents.
pub struct SilverSection {
    // TODO: This should be an enum switching on magic
    /// The magic identifying this section (i.e. 'Str ', 'BMap', 'LDTm', etc.)
    pub section_type: u32,
    // Entries within this section.
    pub entries: Vec<SilverEntry>,
}

/// A high-level representation of entries within a section.
pub struct SilverEntry {
    /// An ID used to identify this entry. For example, 0x0dad06d8.
    pub id: u32,
    // TODO: This should be switchable on magic
    pub contents: Vec<u8>,
}

impl SilverDB {
    pub fn read_file(file: File) -> Result<Self, Error> {
        return SilverDB::read(file);
    }

    pub fn read<T: io::Read + io::Seek>(mut reader: T) -> Result<Self, Error> {
        // First, parse the actual file via binrw.
        let database_file = SilverDBFormat::read(&mut reader)?;

        // Sanity check that should probably be an error:
        assert!(database_file.header.version == 3);
        println!("starting at {}", database_file.remaining_data.len());

        // Next, create the high-level representation.
        let mut sections: Vec<SilverSection> = Vec::new();
        for raw_section in database_file.sections {
            let section_type = raw_section.magic.magic;
            let mut entries: Vec<SilverEntry> = Vec::new();

            for raw_entry in raw_section.entries {
                // We now need to parse data from `remaining_data` ourselves. :(
                //
                // TODO(spotlightishere): Having data being read separately is... messy, to say the least.
                // Perhaps it will be better once https://github.com/jam1garner/binrw/pull/210 is merged.
                let entry_start = raw_entry.data_offset as usize;
                let entry_end = (raw_entry.data_offset + raw_entry.data_size) as usize;
                let raw_entry_data = &database_file.remaining_data[entry_start..entry_end];

                entries.push(SilverEntry {
                    id: raw_entry.id,
                    contents: raw_entry_data.to_vec(),
                });
            }

            // TODO(spotlightishere): Have section type represented by enum
            sections.push(SilverSection {
                section_type,
                entries,
            });
        }

        Ok(SilverDB { sections })
    }
}
