use std::fmt;

use crate::format::SectionMagic;

/// Possible section types.
#[derive(PartialEq)]
pub enum SectionType {
    /// Represents bitmap images within a section ('BMap').
    Bitmap,
    /// Represents UI strings ('Str ').
    String,
    /// Strings used for date/time locale ('LDTm').
    DateTimeLocale,

    /// Not an actual section type - used to represent a currently unknown section's type.
    Unknown(SectionMagic),
}

/// Content represented by sections within.
pub enum SectionContent {
    // TODO(spotlightishere): Images should be parsed accordingly
    Bitmap(Vec<u8>),
    DateTimeLocale(String),
    String(String),

    /// Not an actual section type - used to represent an unknown section's raw binary contents.
    Unknown(Vec<u8>),
}

impl SectionType {
    /// Maps the magic of sections to their enums, based on their little-endian contents.
    pub fn from_magic(magic: SectionMagic) -> Self {
        // The constants below are little-endian values, enforced by binrw.
        match magic {
            // 'Str ' (BE) or ' rtS' (LE)
            [0x20, 0x72, 0x74, 0x53] => SectionType::String,
            // 'LDTm' (BE) or 'mTDL' (LE)
            [0x6D, 0x54, 0x44, 0x4C] => SectionType::DateTimeLocale,
            // 'BMap' (BE) or 'paMB' (LE)
            [0x70, 0x61, 0x4D, 0x42] => SectionType::Bitmap,

            _ => SectionType::Unknown(magic),
        }
    }

    // /// Parses contents to a higher-lvel type accordingly based on their section.
    // fn parse_section(&self, raw_data: Vec<u8>) -> Result<SectionContent, Error> {
    //     match self {
    //         SectionType::String => Ok(SectionContent::String(String::from_utf8(raw_data)?)),
    //     }
    // }
}

impl fmt::Display for SectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            SectionType::Bitmap => "Bitmap".to_string(),
            SectionType::DateTimeLocale => "Date/Time Locale".to_string(),
            SectionType::String => "Strings".to_string(),

            SectionType::Unknown(magic) => {
                // As we parsed in little-endian, we now handle this as big-endian in order to have it be readable.
                let magic_value = u32::from_le_bytes(*magic);
                let magic_string =
                    String::from_utf8(magic.to_vec()).unwrap_or("invalid ASCII magic".to_string());
                format!("{:?} (0x{:08x})", magic_string, magic_value)
            }
        };

        write!(f, "{}", description)?;
        Ok(())
    }
}
