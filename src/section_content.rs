use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{section_types::SectionType, silver_error::SilverError};

/// Content represented by sections within.
#[derive(Deserialize, Serialize)]
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

        let contents: String = Deserialize::deserialize(d)?;
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

/// Similarly to `process_c_string`, when we create C strings once more,
/// they need to end with a null terminator. We simply tack one, where possible.
fn create_c_string(raw_string: String) -> Vec<u8> {
    // TODO(spotlightishere): Using the string's existing bytes doesn't quite work if
    // we're presented a non-ASCII character. Can we, or should we, use a different approach?
    let contents = raw_string.as_bytes();
    let null_terminator: &[u8; 1] = &[0x00];
    [contents, null_terminator].concat()
}

impl SectionContent {
    /// Parses contents to a higher-level type accordingly based on their section.
    pub fn parse_section(
        section_type: &SectionType,
        raw_data: Vec<u8>,
    ) -> Result<SectionContent, SilverError> {
        let section_content = match section_type {
            // TODO(spotlightishere): Handle bitmap parsing
            SectionType::Bitmap => SectionContent::Bitmap(raw_data),
            SectionType::DateTimeLocale => SectionContent::DateTimeLocale(raw_data),
            // Several types are simply C strings.
            SectionType::String
            | SectionType::StringTranslation
            | SectionType::AnimControllerString
            | SectionType::SilverControllerString => {
                SectionContent::String(process_c_string(raw_data)?)
            }
            _ => SectionContent::Unknown(raw_data),
        };
        Ok(section_content)
    }

    /// Reduces contents from their higher-level type to their raw binary representation.
    pub fn reduce_section(section_content: SectionContent) -> Result<Vec<u8>, SilverError> {
        let raw_data = match section_content {
            // TODO(spotlightishere): Handle bitmap parsing
            SectionContent::Bitmap(raw_contents) => raw_contents,
            SectionContent::DateTimeLocale(raw_contents) => raw_contents,
            SectionContent::Unknown(raw_contents) => raw_contents,
            SectionContent::String(raw_string) => create_c_string(raw_string),
        };

        Ok(raw_data)
    }
}
