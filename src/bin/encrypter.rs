use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file_to_encrypt>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];

    match fs::read_to_string(filename) {
        Ok(content) => {
            if content.is_empty() {
                println!("The file '{}' is empty.", filename);
            } else {
                println!("Successfully read file '{}'.", filename);
                // Encryption logic will go here in the future.
            }
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    }
}
