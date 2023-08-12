use std::path::Path;

use crate::database::SilverDB;

pub fn serialize_contents(database: SilverDB) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Allow customizing output path and generally fix up
    let output_dir = Path::new("./output");
    if output_dir.exists() {
        std::fs::remove_dir_all(output_dir)?;
    }
    std::fs::create_dir(output_dir)?;

    for current_section in database.sections {
        // TODO(spotlightishere): This is copied from the formatter for SectionType.
        // It should really be avoided altogether.
        let section_string = current_section.section_type.to_string();
        let file_name = format!("{section_string}.yaml");
        let output_file = output_dir.join(Path::new(&file_name));

        let all_contents = serde_yaml::to_string(&current_section.resources)?;
        std::fs::write(output_file, all_contents)?;
    }

    Ok(())
}
