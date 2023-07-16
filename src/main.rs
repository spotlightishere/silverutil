use database::SilverDB;
use std::{env, fs::File};

mod database;
mod format;

fn main() {
    // TODO: Implement proper flags eventually
    // Possibly via clap?
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() != 3 {
        println!("Incorrect usage!");
        println!(
            "Usage: {} [info|extract] path/to/silverdb.bin",
            arguments[0]
        );
        return;
    }

    let operation = &arguments[1];
    let database_path = &arguments[2];
    if operation != "info" {
        unimplemented!()
    }

    // Parse our file!
    let database_file = File::open(database_path).expect("unable to open SilverDB database");
    let database = SilverDB::read_file(database_file).expect("unable to parse SilverDB database");

    println!("Found valid SilverDB database!");
    println!("There are {} sections.", database.sections.len());
    println!("Sections:");
    let sections = database.sections;
    for section in &sections {
        println!("-------------------------------------------");
        println!("Section: {}", section.section_type);
        println!("\tEntry count: {}", section.entries.len());
        println!("\tEntries:");
        for entry in &section.entries {
            println!("\t\t- Entry ID: {:08x}", entry.id);
            println!("\t\t  Size: {}", entry.contents.len());
        }
        println!("-------------------------------------------");
    }

    // Test: Print all 'Str ' types
    let string_section = sections
        .as_slice()
        .iter()
        .find(|&section| section.section_type == 0x53747220)
        .expect("unable to find strings section");
    for string_entry in string_section.entries.as_slice() {
        println!(
            "String entry unadjusted offset: {}",
            String::from_utf8(string_entry.contents.to_owned()).expect("should be a string")
        )
    }
}
