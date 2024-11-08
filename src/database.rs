use serde::{Deserialize, Serialize};
use std::{fmt, io::Cursor};

use crate::{
    format::SilverDBFormat,
    format::{ResourceMetadata, SectionHeader, SilverDBHeader},
    section_content::SectionContent,
    section_types::SectionType,
    silver_error::SilverError,
};

/// A high-level representation of a SilverDB file.
pub struct SilverDB {
    /// Available sections within this database.
    pub sections: Vec<SilverSection>,
}

/// A high-level representation of section contents.
pub struct SilverSection {
    /// The magic identifying this section (i.e. 'Str ', 'BMap', 'LDTm', etc.)
    pub section_type: SectionType,

    /// Whether this section's IDs are sequential and start at 1.
    /// Necessary for certain resource types such as `StrT`.
    pub is_sequential: u32,

    // Resources within this section.
    pub resources: Vec<SilverResource>,
}

/// The ID identifying this resource.
/// In general you should never modify the ID as it may be hardcoded in firmware.
#[derive(Deserialize, Serialize)]
pub struct SilverResourceID(pub u32);

impl fmt::Display for SilverResourceID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}

impl From<u32> for SilverResourceID {
    fn from(value: u32) -> Self {
        SilverResourceID(value)
    }
}

/// A high-level representation of resources within a section.
#[derive(Deserialize, Serialize)]
pub struct SilverResource {
    /// An ID used to identify this resources. For example, 0x0dad06d8.
    pub id: SilverResourceID,

    /// The content this resource holds.
    pub contents: SectionContent,
}

impl SilverDB {
    pub fn read(file_contents: Vec<u8>) -> Result<Self, SilverError> {
        // First, parse the actual file.
        let reader = Cursor::new(file_contents);
        let database_file = SilverDBFormat::read(reader)?;

        // Next, create the high-level representation.
        let mut sections: Vec<SilverSection> = Vec::new();
        for raw_section in database_file.sections {
            let section_type = SectionType::from_magic(raw_section.magic);
            let mut resources: Vec<SilverResource> = Vec::new();

            for raw_resource in raw_section.resources {
                // TODO(spotlightishere): Have section contents parsed accordingly
                let contents = SectionContent::parse_section(&section_type, raw_resource.contents)?;

                resources.push(SilverResource {
                    id: SilverResourceID(raw_resource.id),
                    contents,
                });
            }

            let is_sequential = raw_section.is_sequential;
            sections.push(SilverSection {
                section_type,
                is_sequential,
                resources,
            });
        }

        Ok(SilverDB { sections })
    }

    pub fn write(all_sections: Vec<SilverSection>) -> Result<Vec<u8>, SilverError> {
        // First, we need to reduce the high-level representations to their binary formats.
        let mut raw_sections: Vec<SectionHeader> = Vec::new();

        for current_section in all_sections {
            // We need to synthesize resource metadata for all resouces within this section.
            let mut all_resources: Vec<ResourceMetadata> = Vec::new();
            for current_resource in current_section.resources {
                // We reduce this section back to its raw, Vec<u8> form.
                let raw_resource = SectionContent::reduce_section(current_resource.contents)?;

                let resource = ResourceMetadata {
                    id: current_resource.id.0,
                    // This will be filled in by binrw when writing.
                    data_offset: 0,
                    data_size: raw_resource.len() as u32,
                    contents: raw_resource,
                };
                all_resources.push(resource);
            }

            // Lastly, we synthesize section header information.
            let raw_section = SectionHeader {
                magic: current_section.section_type.to_magic(),
                // TODO(spotlightishere): Should binrw calculate this for us?
                // It's simple enough to figure out manually, thankfully.
                resource_count: all_resources.len() as u32,
                is_sequential: current_section.is_sequential,
                // This will be filled in by binrw when writing.
                resource_offset: 0,
                resources: all_resources,
            };
            raw_sections.push(raw_section);
        }

        // Next, we create a mock SilverDB struct. This is simply to give our resources.
        let mock_database = SilverDBFormat {
            header: SilverDBHeader {
                // 0x03 across 5th, 6th, and 7th generation iPod nanos.
                version: 3,
                // This will be filled in by binrw when writing.
                header_length: 0,
                section_count: raw_sections.len() as u32,
            },
            sections: raw_sections,
        };

        let raw_contents = mock_database.write()?;
        Ok(raw_contents)
    }
}
