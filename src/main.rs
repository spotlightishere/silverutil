use database::SilverDB;
use marshal::serialize_contents;
use std::{env, fs::File};

use crate::section_content::SectionContent;
use crate::section_types::SectionType;

mod database;
mod format;
mod marshal;
mod section_content;
mod section_types;

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

    // Parse our file!
    let database_file = File::open(database_path).expect("unable to open SilverDB database");
    let database = SilverDB::read_file(database_file).expect("unable to parse SilverDB database");

    if operation == "info" {
        info(database);
        return;
    } else if operation != "extract" {
        panic!("Invalid operation!")
    }
    serialize_contents(database).expect("failed to serialize");
}

fn info(database: SilverDB) {
    println!("There are {} sections.", database.sections.len());
    println!("Sections:");
    let sections = database.sections;
    for section in &sections {
        println!("-------------------------------------------");
        println!("Section: {}", section.section_type);
        println!("\tResource count: {}", section.resources.len());
        println!("\tResources:");
        for resource in &section.resources {
            println!("\t\t- Resource ID: 0x{:08x}", resource.id);
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
                println!("String resource (0x{:08x}): {}", &string_resource.id, value)
            }
            _ => println!("Unknown content type!"),
        }
    }
}
