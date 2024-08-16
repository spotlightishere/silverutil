use std::fs;
use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use silverlib::{
    RawBitmapType, SectionContent, SectionType, SilverDB, SilverResource, SilverSection,
};

#[derive(Deserialize, Serialize)]
/// Generic section metadata.
pub struct SectionMetadata {
    magic: SectionType,
    is_sequential: u32,
    resources: Vec<SilverResource>,
}

#[derive(Deserialize, Serialize)]
/// Bitmap-specific section metadata.
pub struct BitmapMetadata {
    magic: SectionType,
    is_sequential: u32,
    resources: Vec<BitmapImageMetadata>,
}

#[derive(Deserialize, Serialize)]
/// Bitmap-specific entry metadata.
pub struct BitmapImageMetadata {
    pub width: u32,
    pub height: u32,
    pub format_type: RawBitmapType,
    pub resource_id: u32,
    pub path: String,
}

type AnyError = Box<dyn std::error::Error>;

pub fn serialize_contents(database: SilverDB, output_dir: &Path) -> Result<(), AnyError> {
    // TODO(spotlightishere): Should we blindly obliterate the output directory?
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }
    fs::create_dir(output_dir)?;

    // As we process each section, take note of its section metadata.
    // This will be parsed in order to ensure sections are in order.
    let mut section_list: Vec<String> = Vec::new();

    for current_section in database.sections {
        let section_name = current_section.section_type.to_name();
        let file_name = format!("{section_name}.yaml");
        let output_file = output_dir.join(Path::new(&file_name));

        // For the majority of sections, we create a custom "SectionMetadata"
        // type in order to persist this section's magic and flags.
        // However, we must handle bitmap images separately.
        match current_section.section_type {
            SectionType::Bitmap | SectionType::StatusBarBitmap => {
                // Keep track of written bitmap image entries.
                let mut bitmap_list = Vec::new();

                // The directory we're outputting our PNGs within.
                let section_dir = output_dir.join(section_name.clone());
                fs::create_dir(section_dir.clone())?;

                // We'll need to write every bitmap image to disk.
                for bitmap_entry in current_section.resources {
                    // Ensure we truly have a bitmap representation.
                    let SectionContent::Bitmap(entry_contents) = bitmap_entry.contents else {
                        panic!("expected bitmap image type to have bitmap contents");
                    };

                    // Some bitmap images have a resource ID but entirely lack any data.
                    // If we have a 0x0 image and the special path "empty", assume that this is the case.
                    // TODO(spotlightishere): This is absurdly messy.
                    let resource_id = bitmap_entry.id.0;
                    let Some(entry_contents) = entry_contents else {
                        let empty_metadata = BitmapImageMetadata {
                            width: 0,
                            height: 0,
                            // This is not actually the format type, as it has none.
                            format_type: RawBitmapType::Rgb565,
                            resource_id,
                            path: "empty".to_string(),
                        };
                        bitmap_list.push(empty_metadata);
                        continue;
                    };

                    // Write out our image.
                    let output_relative = format!("{}/{}.png", section_name, resource_id);
                    let output_path = section_dir.join(format!("{}.png", resource_id));
                    fs::write(output_path, entry_contents.contents)?;

                    let entry_metadata = BitmapImageMetadata {
                        width: entry_contents.width,
                        height: entry_contents.height,
                        format_type: entry_contents.format_type,
                        resource_id: entry_contents.resource_id,
                        path: output_relative,
                    };
                    bitmap_list.push(entry_metadata);
                }

                let bitmap_metadata = BitmapMetadata {
                    magic: current_section.section_type,
                    is_sequential: current_section.is_sequential,
                    resources: bitmap_list,
                };
                let all_contents = serde_yaml::to_string(&bitmap_metadata)?;
                fs::write(output_file, all_contents)?;
            }
            // Otherwise, simply write out a raw representation of the section's metadata.
            _ => {
                let section_metadata = SectionMetadata {
                    magic: current_section.section_type,
                    is_sequential: current_section.is_sequential,
                    resources: current_section.resources,
                };
                let all_contents = serde_yaml::to_string(&section_metadata)?;
                fs::write(output_file, all_contents)?;
            }
        }

        section_list.push(section_name);
    }

    // Finally, write our section metadata.
    let metadata = serde_yaml::to_string(&section_list)?;
    let metadata_path = output_dir.join(Path::new("metadata.yaml"));
    fs::write(metadata_path, metadata)?;

    Ok(())
}

/// Deserializes the YAML at the given path to the inferred type.
fn read_yaml<T>(input_path: &PathBuf) -> Result<T, AnyError>
where
    T: DeserializeOwned,
{
    let yaml_file = fs::File::open(input_path)?;
    let unmarshalled: T = serde_yaml::from_reader(yaml_file)?;
    Ok(unmarshalled)
}

pub fn deserialize_contents(input_dir: &Path, database_path: &Path) -> Result<(), AnyError> {
    // First, load section metadata.
    let metadata_path = input_dir.join(Path::new("metadata.yaml"));
    let section_list: Vec<String> = read_yaml(&metadata_path)?;

    let mut all_sections: Vec<SilverSection> = Vec::new();

    for section_name in section_list {
        // For every metadata section, parse its respective YAML representation.
        let file_name = format!("{}.yaml", section_name);
        let section_path = input_dir.join(Path::new(&file_name));
        let section_contents: SectionMetadata = read_yaml(&section_path)?;

        // TODO(spotlightishere): Implement
        if section_contents.magic == SectionType::Bitmap
            || section_contents.magic == SectionType::StatusBarBitmap
        {
            todo!("deserialization of bitmap contents is not currently implemented");
        }

        let current_section = SilverSection {
            section_type: section_contents.magic,
            is_sequential: section_contents.is_sequential,
            resources: section_contents.resources,
        };
        all_sections.push(current_section);
    }

    // Finally, write our raw database.
    let raw_database = SilverDB::write(all_sections)?;
    fs::write(database_path, raw_database)?;
    Ok(())
}
