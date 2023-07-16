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
    for section in database.sections {
        println!("-------------------------------------------");
        println!("Section: {}", section.magic);
        println!("\tFile count: {}", section.file_count);
        println!("-------------------------------------------");
    }
}
