use std::{fs::File, io, string::FromUtf8Error};

use binrw::BinRead;

use crate::{
    format::SilverDBFormat,
    sections::{SectionContent, SectionType},
};

/// Possible errors encountered when parsing, or etc.
#[derive(Debug)]
pub enum SilverError {
    ContentParseFailure(FromUtf8Error),
    InvalidVersion,
    ParseError(binrw::Error),
}

impl From<binrw::Error> for SilverError {
    fn from(value: binrw::Error) -> Self {
        SilverError::ParseError(value)
    }
}

// TODO(spotlightishere): This will not scale well as other content types are parsed.
impl From<FromUtf8Error> for SilverError {
    fn from(value: FromUtf8Error) -> Self {
        SilverError::ContentParseFailure(value)
    }
}

/// A high-level representation of a SilverDB file.
pub struct SilverDB {
    /// Available sections within this database.
    pub sections: Vec<SilverSection>,
}

/// A high-level representation of section contents.
pub struct SilverSection {
    // TODO: This should be an enum switching on magic
    /// The magic identifying this section (i.e. 'Str ', 'BMap', 'LDTm', etc.)
    pub section_type: SectionType,

    // Resources within this section.
    pub resources: Vec<SilverResource>,
}

/// A high-level representation of resources within a section.
pub struct SilverResource {
    /// An ID used to identify this resources. For example, 0x0dad06d8.
    pub id: u32,
    /// The content this resource holds.
    pub contents: SectionContent,
}

impl SilverDB {
    pub fn read_file(file: File) -> Result<Self, SilverError> {
        return SilverDB::read(file);
    }

    pub fn read<T: io::Read + io::Seek>(mut reader: T) -> Result<Self, SilverError> {
        // First, parse the actual file via binrw.
        let database_file = SilverDBFormat::read(&mut reader)?;

        // Sanity check:
        if database_file.header.version != 3 {
            return Err(SilverError::InvalidVersion);
        }

        // Next, create the high-level representation.
        let mut sections: Vec<SilverSection> = Vec::new();
        for raw_section in database_file.sections {
            let section_type = SectionType::from_magic(raw_section.magic);
            let mut resources: Vec<SilverResource> = Vec::new();

            for raw_resource in raw_section.resources {
                // TODO(spotlightishere): Have section contents parsed accordingly
                let contents = section_type.parse_section(raw_resource.contents)?;

                resources.push(SilverResource {
                    id: raw_resource.id,
                    contents,
                });
            }

            // TODO(spotlightishere): Have section type represented by enum
            sections.push(SilverSection {
                section_type,
                resources,
            });
        }

        Ok(SilverDB { sections })
    }
}
