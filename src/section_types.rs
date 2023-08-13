use crate::format::SectionMagic;
use std::fmt;

/// Possible known section types.
#[derive(PartialEq)]
pub enum SectionType {
    /// Represents bitmap images within a section ('BMap').
    Bitmap,
    /// Represents UI strings ('Str ').
    String,
    /// Represents placeholders for view strings. ('StrT')
    // TODO(spotlightishere): This name likely isn't correct - please fix up!
    StringTranslation,
    /// Strings used for date/time locale ('LDTm').
    DateTimeLocale,
    /// Names of animation controller C++ classes as strings
    AnimControllerString,
    /// Names of UI controller C++ classes as strings
    SilverControllerString,

    /// Not an actual section type - used to represent a currently unknown section's type.
    Unknown(SectionMagic),
}

impl SectionType {
    /// Maps the magic of sections to their enums, based on their little-endian contents.
    pub fn from_magic(magic: SectionMagic) -> Self {
        // The constants below are little-endian values, enforced by binrw.
        match magic {
            // 'Str ' (BE) or ' rtS' (LE)
            [0x20, 0x72, 0x74, 0x53] => SectionType::String,
            // 'StrT' (BE) or 'TrtS' (LE)
            [0x54, 0x72, 0x74, 0x53] => SectionType::StringTranslation,
            // 'LDTm' (BE) or 'mTDL' (LE)
            [0x6D, 0x54, 0x44, 0x4C] => SectionType::DateTimeLocale,
            // 'BMap' (BE) or 'paMB' (LE)
            [0x70, 0x61, 0x4D, 0x42] => SectionType::Bitmap,
            // 'ACST' (BE) or 'TSCA' (LE)
            [0x54, 0x53, 0x43, 0x41] => SectionType::AnimControllerString,
            // 'SCST' (BE) or 'TSCS' (LE)
            [0x54, 0x53, 0x43, 0x53] => SectionType::SilverControllerString,
            _ => SectionType::Unknown(magic),
        }
    }

    /// Maps the current enum section type to its magic value.
    pub fn to_magic(&self) -> SectionMagic {
        // Similar to `from_magic` above, the constants below are
        // little-endian values, as enforced by binrw.
        match self {
            // 'Str ' (BE) or ' rtS' (LE)
            SectionType::String => [0x20, 0x72, 0x74, 0x53],
            // 'StrT' (BE) or 'TrtS' (LE)
            SectionType::StringTranslation => [0x54, 0x72, 0x74, 0x53],
            // 'LDTm' (BE) or 'mTDL' (LE)
            SectionType::DateTimeLocale => [0x6D, 0x54, 0x44, 0x4C],
            // 'BMap' (BE) or 'paMB' (LE)
            SectionType::Bitmap => [0x70, 0x61, 0x4D, 0x42],
            // 'ACST' (BE) or 'TSCA' (LE)
            SectionType::AnimControllerString => [0x54, 0x53, 0x43, 0x41],
            // 'SCST' (BE) or 'TSCS' (LE)
            SectionType::SilverControllerString => [0x54, 0x53, 0x43, 0x53],

            SectionType::Unknown(magic) => *magic,
        }
    }

    /// Converts the current enum to its four-byte string.
    pub fn to_string(&self) -> String {
        // As our mapped values are little-endian, we now handle this as
        // big-endian in order to have it be readable.
        let magic_value = u32::from_le_bytes(self.to_magic());
        let big_endian_value = magic_value.to_be_bytes().to_vec();
        String::from_utf8(big_endian_value).expect("invalid ASCII magic")
    }
}

impl fmt::Display for SectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let temp_magic;

        let description = match self {
            SectionType::AnimControllerString => "Animation Controller Strings",
            SectionType::Bitmap => "Bitmap",
            SectionType::DateTimeLocale => "Date/Time Locale",
            SectionType::SilverControllerString => "Silver UI Controller Strings",
            SectionType::String => "Strings",
            SectionType::StringTranslation => "String View Placeholder",

            SectionType::Unknown(magic) => {
                let magic_value = u32::from_le_bytes(*magic);
                temp_magic = format!("{:?} (0x{:02X})", self.to_string(), magic_value);
                temp_magic.as_str()
            }
        };

        write!(f, "{}", description)?;
        Ok(())
    }
}
