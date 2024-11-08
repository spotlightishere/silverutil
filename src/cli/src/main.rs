use clap::{Parser, Subcommand};

use std::{fs::File, io::Read, path::PathBuf};

use silverlib::{SectionContent, SectionType, SilverDB};

mod marshal;
mod scrape;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Extracts sections within database into a YAML representation
    Extract {
        /// Path to Silver database to extract
        database_path: PathBuf,
        /// Directory to output YAML representations within
        output_dir: PathBuf,
    },
    /// Displays information about contents present within sections
    Info { database_path: PathBuf },
    /// Creates a database from a YAML representation
    Create {
        /// Directory holding the YAML representations to create from
        input_dir: PathBuf,
        /// Path to write Silver databases to
        database_path: PathBuf,
    },
    /// Scrapes SilverDBs embedded within a given firmware file
    Scrape {
        /// Path to the retailOS firmware to scrape databases from.
        firmware_path: PathBuf,
        /// Directory to output SilverDBs within
        output_dir: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Subcommands::Extract {
            database_path,
            output_dir,
        } => {
            let database = open_database(database_path);
            marshal::serialize_contents(database, &output_dir)
                .expect("failed to serialize database to YAML representation");
        }
        Subcommands::Info { database_path } => {
            let database = open_database(database_path);
            print_info(database)
        }
        Subcommands::Create {
            input_dir,
            database_path,
        } => {
            marshal::deserialize_contents(&input_dir, &database_path)
                .expect("failed to deserialize YAML representation");
        }
        Subcommands::Scrape {
            firmware_path,
            output_dir,
        } => scrape::handle_scrape(&firmware_path, &output_dir)
            .expect("failed to scrape resource databases within firmware"),
    };
}

/// Parses the given path.
fn open_database(database_path: PathBuf) -> SilverDB {
    let mut database_file = File::open(database_path).expect("unable to open SilverDB database");
    let mut file_contents: Vec<u8> = Vec::new();
    database_file
        .read_to_end(&mut file_contents)
        .expect("unable to read contents of SilverDB database");

    SilverDB::read(file_contents).expect("unable to parse SilverDB database")
}

fn print_info(database: SilverDB) {
    println!("There are {} sections.", database.sections.len());
    println!("Sections:");
    let sections = database.sections;
    for section in &sections {
        println!("-------------------------------------------");
        println!("Section: {}", section.section_type);
        println!("\tResource count: {}", section.resources.len());
        println!("\tResources:");
        for resource in &section.resources {
            println!("\t\t- Resource ID: {}", resource.id);
        }
        println!("-------------------------------------------");
    }

    // Test: Print all 'Str ' types
    let string_section = sections
        .as_slice()
        .iter()
        .find(|&section| section.section_type == SectionType::String);
    let Some(string_section) = string_section else {
        println!("No string section found in this database.");
        return;
    };

    for string_resource in string_section.resources.as_slice() {
        match &string_resource.contents {
            SectionContent::String(value) => {
                println!("String resource ({}): {}", &string_resource.id, value)
            }
            _ => println!("Unknown content type!"),
        }
    }
}
