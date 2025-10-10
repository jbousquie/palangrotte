//! # Encrypter Utility
//! This binary provides a command-line utility for encrypting and decrypting files.
//! It uses the encryption functions from the `palangrotte` library.

use palangrotte::encryption::{encrypt_file, decrypt_file, EncryptedFile, PBKDF2_SALT_LEN};
use ring::aead::NONCE_LEN;
use std::fs;
use std::io::{Read, Write};
use std::env;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;


/// The main function for the encrypter utility.
///
/// This function parses command-line arguments to determine whether to encrypt or decrypt a file.
/// It prompts the user for a password and then performs the requested operation.
///
/// # Arguments
///
/// * `<encrypt|decrypt>` - The command to perform.
/// * `<input_file>` - The path to the input file.
/// * `<output_file>` - The path to the output file.
///
/// # Returns
///
/// * `Ok(())` - If the operation was successful.
/// * `Err(Box<dyn std::error::Error>)` - If there was an error.
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <encrypt|decrypt> <input_file> <output_file>", args[0]);
        return Ok(());
    }

    let command = &args[1];
    let input_path = &args[2];
    let output_path = &args[3];

    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist.", input_path);
        return Ok(());
    }

    match command.as_str() {
        "encrypt" => {
            let password = rpassword::prompt_password("Enter password: ")?;
            let password_confirm = rpassword::prompt_password("Confirm password: ")?;

            if password != password_confirm {
                eprintln!("Passwords do not match.");
                return Ok(());
            }

            let plaintext = fs::read(input_path)?;
            match encrypt_file(&plaintext, &password) {
                Ok(enc_data) => {
                    let mut file = fs::File::create(output_path)?;
                    file.write_all(&enc_data.salt)?;
                    file.write_all(&enc_data.nonce)?;
                    file.write_all(&enc_data.ciphertext_with_tag)?;
                    println!("File encrypted successfully to: {}", output_path);
                }
                Err(_) => {
                    eprintln!("Error during file encryption.");
                }
            }
        }
        "decrypt" => {
            let password = rpassword::prompt_password("Enter password: ")?;
            let mut encrypted_file = fs::File::open(input_path)?;
            let mut salt = [0u8; PBKDF2_SALT_LEN];
            encrypted_file.read_exact(&mut salt)?;
            let mut nonce = [0u8; NONCE_LEN];
            encrypted_file.read_exact(&mut nonce)?;
            let mut ciphertext_with_tag = Vec::new();
            encrypted_file.read_to_end(&mut ciphertext_with_tag)?;

            let read_enc_data = EncryptedFile {
                salt,
                nonce,
                ciphertext_with_tag,
            };

            match decrypt_file(read_enc_data, &password) {
                Ok(decrypted) => {
                    fs::write(output_path, decrypted)?;
                    println!("File decrypted successfully to: {}", output_path);
                }
                Err(_) => {
                    eprintln!("Error during file decryption. Incorrect password or corrupted data.");
                }
            }
        }
        _ => {
            eprintln!("Invalid command. Use 'encrypt' or 'decrypt'.");
        }
    }

    Ok(())
}