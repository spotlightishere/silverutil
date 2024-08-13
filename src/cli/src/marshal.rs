use std::fs;
use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use silverlib::{SectionType, SilverDB, SilverResource, SilverSection};

#[derive(Deserialize, Serialize)]
pub struct SectionMetadata {
    magic: SectionType,
    is_sequential: u32,
    resources: Vec<SilverResource>,
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
        // TODO(spotlightishere): SectionType should (de)serialize into a string.
        let section_string = current_section.section_type.to_string();
        let file_name = format!("{section_string}.yaml");
        let output_file = output_dir.join(Path::new(&file_name));

        // We create a custom "SectionMetadata" type in order to
        // persist this section's magic and flags.
        let section_metadata = SectionMetadata {
            magic: current_section.section_type,
            is_sequential: current_section.is_sequential,
            resources: current_section.resources,
        };
        let all_contents = serde_yaml::to_string(&section_metadata)?;
        fs::write(output_file, all_contents)?;

        section_list.push(section_string);
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
