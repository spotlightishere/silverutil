use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{database::SilverError, section_types::SectionType};

/// Content represented by sections within.
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum SectionContent {
    // TODO(spotlightishere): Images should be parsed accordingly
    #[serde(with = "RawData")]
    Bitmap(Vec<u8>),

    #[serde(with = "RawData")]
    DateTimeLocale(Vec<u8>),

    /// A generic string type. Handled as a C string.
    String(String),

    /// Not an actual section type - used to represent an unknown section's raw binary contents.
    #[serde(with = "RawData")]
    Unknown(Vec<u8>),
}

/// Generic, arbitrary data.
struct RawData;
impl RawData {
    fn serialize<S>(contents: &Vec<u8>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(hex::encode(contents).as_str())
    }

    fn deserialize<'de, D>(d: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let contents: &str = Deserialize::deserialize(d)?;
        let raw_data = hex::decode(contents).map_err(Error::custom)?;
        Ok(raw_data)
    }
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
            SectionType::StringTranslation => SectionContent::String(process_c_string(raw_data)?),
            SectionType::AnimControllerString => {
                SectionContent::String(process_c_string(raw_data)?)
            }
            SectionType::SilverControllerString => {
                SectionContent::String(process_c_string(raw_data)?)
            }
            _ => SectionContent::Unknown(raw_data),
        };
        Ok(section_content)
    }
}
