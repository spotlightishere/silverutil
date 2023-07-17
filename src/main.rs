use database::SilverDB;
use std::{env, fs::File};

use crate::sections::SectionType;

mod database;
mod format;
mod sections;

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
        println!("\tResource count: {}", section.resources.len());
        println!("\tResources:");
        for resource in &section.resources {
            println!("\t\t- Resource ID: {:08x}", resource.id);
            println!("\t\t  Size: {}", resource.contents.len());
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
        println!(
            "String resource: {}",
            String::from_utf8(string_resource.contents.to_owned()).expect("should be a string")
        )
    }
}
