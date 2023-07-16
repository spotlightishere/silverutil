use binrw::BinRead;
use format::SilverDB;
use std::{env, fs::File};

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
    let mut database_file = File::open(database_path).expect("unable to open SilverDB database");
    let database = SilverDB::read(&mut database_file).expect("unable to parse SilverDB database");
    if database.header.version != 3 {
        panic!("Unknown database version!");
    }

    println!("Found valid SilverDB database!");
    println!("There are {} sections.", database.header.section_count);
    println!("Sections:");
    let sections = &database.sections;
    for section in sections {
        println!("-------------------------------------------");
        println!("Section: {}", section.magic);
        println!("\tEntry count: {}", section.entry_count);
        println!("\tEntries:");
        for entry in &section.entries {
            println!("\t\t- Entry ID: {:08x}", entry.id);
            println!("\t\t  Size: {}", entry.data_size);
            println!("\t\t  Offset: {}", entry.data_offset);
        }
        println!("-------------------------------------------");
    }

    // Test: Print all 'Str ' types
    let string_section = sections
        .as_slice()
        .iter()
        .find(|&section| section.magic.magic == 0x53747220)
        .expect("unable to find strings section");
    for string_entry in string_section.entries.as_slice() {
        println!(
            "String entry unadjusted offset: {}",
            string_entry.data_offset
        )
    }
}
