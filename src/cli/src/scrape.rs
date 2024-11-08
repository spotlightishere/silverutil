use silverlib::{SilverDB, SilverDBFormat, SilverError};
use std::{fs, io::Cursor, path::Path};

use crate::marshal;

/// Metadata about SilverDBs found throughout the search.
pub struct ScrapeMetadata {
    pub offset: u32,
    pub database: SilverDB,
}

type AnyError = Box<dyn std::error::Error>;

pub fn handle_scrape(firmware_path: &Path, output_dir: &Path) -> Result<(), AnyError> {
    // TODO(spotlightishere): We really should not obliterate the output directory.
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }
    fs::create_dir(output_dir)?;

    // Load our firmware entirely to a Vec<u8>.
    // This is a terribly inefficient use of resources.
    let firmware_contents = fs::read(firmware_path)?;

    // Our running metadata.
    let mut found_databases: Vec<ScrapeMetadata> = Vec::new();

    // Search for SilverDBs with version 3.
    // This is its representation as a little endian uint32_t.
    const FIRMWARE_MAGIC: [u8; 4] = [0x03, 0x00, 0x00, 0x00];
    let found_offsets = firmware_contents
        .windows(4)
        .enumerate()
        .filter(|(_, x)| *x == FIRMWARE_MAGIC)
        .map(|(index, _)| index);

    for current_offset in found_offsets {
        // Seek to the found offset.
        //
        // TODO(spotlightishere): We should really be seeking relatively:
        // our LittleHelper directly leverages `set_position`, breaking
        // the ability to pass a Cursor over this larger Vec<u8>.
        let smaller_firmware = &firmware_contents[current_offset..];

        // Try to load only the database header for validation.
        let header_cursor = Cursor::new(Vec::from(smaller_firmware));
        _ = match SilverDBFormat::read(header_cursor) {
            // This isn't a valid SilverDB.
            Err(SilverError::InvalidHeader) => continue,
            Err(e) => panic!("failed to parse scraped SilverDB: {}", e),
            Ok(db) => db,
        };

        println!("Found a database at offset {}...", current_offset);

        // Attempt to load this as a database.
        let firmware_cursor = Cursor::new(Vec::from(smaller_firmware));
        let database = match SilverDB::read_cursor(firmware_cursor) {
            // This isn't a valid SilverDB.
            Err(SilverError::InvalidHeader) => continue,
            Err(e) => panic!("failed to parse scraped SilverDB: {}", e),
            Ok(db) => db,
        };

        // If we were successful, track this.
        let metadata = ScrapeMetadata {
            offset: current_offset as u32,
            database,
        };
        found_databases.push(metadata);
    }

    // We only need to process
    if found_databases.is_empty() {
        println!("Found no SilverDB databases within this firmware.");
        return Ok(());
    }

    // Marshal every found database.
    for database_metadata in found_databases {
        let offset_string = database_metadata.offset.to_string();
        println!("Extracting database at offset {}...", offset_string);

        // e.g. "./output/5974312"
        let offset_dir = format!("{offset_string}/");
        let output_dir = output_dir.join(offset_dir);

        marshal::serialize_contents(database_metadata.database, &output_dir)?;
    }

    Ok(())
}
