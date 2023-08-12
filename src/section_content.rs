use serde::{Serialize, Serializer};

use crate::{database::SilverError, section_types::SectionType};

/// Content represented by sections within.
#[derive(Serialize)]
#[serde(untagged)]
pub enum SectionContent {
    // TODO(spotlightishere): Images should be parsed accordingly
    #[serde(serialize_with = "serialize_bytes")]
    Bitmap(Vec<u8>),

    #[serde(serialize_with = "serialize_bytes")]
    DateTimeLocale(Vec<u8>),

    String(String),
    StringTranslation(String),

    /// Not an actual section type - used to represent an unknown section's raw binary contents.
    #[serde(serialize_with = "serialize_bytes")]
    Unknown(Vec<u8>),
}

fn serialize_bytes<S>(contents: &Vec<u8>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(hex::encode(contents).as_str())
}

/// As many resources are C strings, they contain a null terminator.
/// We process them by removing the trailing null terminator.
fn process_c_string(raw_data: Vec<u8>) -> Result<std::string::String, SilverError> {
    let mut contents = String::from_utf8(raw_data)?;
    // Remove the last byte, a null byte.
    contents.truncate(contents.len() - 1);
    Ok(contents)
}

impl SectionContent {
    /// Parses contents to a higher-level type accordingly based on their section.
    pub fn parse_section(
        section_type: &SectionType,
        raw_data: Vec<u8>,
    ) -> Result<SectionContent, SilverError> {
        let section_content = match section_type {
            SectionType::Bitmap => SectionContent::Bitmap(raw_data),
            SectionType::DateTimeLocale => SectionContent::DateTimeLocale(raw_data),
            SectionType::String => SectionContent::String(process_c_string(raw_data)?),
            SectionType::StringTranslation => {
                SectionContent::StringTranslation(String::from_utf8(raw_data)?)
            }
            _ => SectionContent::Unknown(raw_data),
        };
        Ok(section_content)
    }
}
