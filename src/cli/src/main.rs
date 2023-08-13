use clap::{Parser, Subcommand};

use std::{fs::File, path::PathBuf};

use silverlib::{SectionContent, SectionType, SilverDB};

mod marshal;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Subcommands>,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Extracts sections within binary into a YAML representation
    Extract {
        /// Path to Silver database to extract
        database_path: PathBuf,
        /// Directory to output YAML representation within
        output_dir: PathBuf,
    },
    /// Displays information about contents present within sections
    Info { database_path: PathBuf },
}

fn main() {
    let cli = Cli::parse();
    // We should not have optional subcommands.
    let subcommand = cli.command.unwrap();

    match subcommand {
        Subcommands::Extract {
            database_path,
            output_dir,
        } => {
            let database = open_database(database_path);
            marshal::serialize_contents(database, &output_dir).expect("failed to serialize");
        }
        Subcommands::Info { database_path } => {
            let database = open_database(database_path);
            print_info(database)
        }
    };
}

/// Parses the given path.
fn open_database(database_path: PathBuf) -> SilverDB {
    let database_file = File::open(database_path).expect("unable to open SilverDB database");
    SilverDB::read_file(database_file).expect("unable to parse SilverDB database")
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
        .find(|&section| section.section_type == SectionType::String)
        .expect("unable to find strings section");
    for string_resource in string_section.resources.as_slice() {
        match &string_resource.contents {
            SectionContent::String(value) => {
                println!("String resource ({}): {}", &string_resource.id, value)
            }
            _ => println!("Unknown content type!"),
        }
    }
}
