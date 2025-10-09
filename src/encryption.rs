use ring::{
    aead::{self, Nonce, UnboundKey, LessSafeKey, CHACHA20_POLY1305, NONCE_LEN},
    error::Unspecified,
    pbkdf2,
    rand::{SecureRandom, SystemRandom},
};
use std::num::NonZeroU32;

// --- Cryptographic Constants ---
pub const PBKDF2_ITERATIONS: u32 = 100_000;
pub const PBKDF2_SALT_LEN: usize = 16;

// Data structure to save: Salt + Nonce + Ciphertext (including Tag)
#[derive(Debug)]
pub struct EncryptedFile {
    pub salt: [u8; PBKDF2_SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
    pub ciphertext_with_tag: Vec<u8>,
}

// --- Encryption and Decryption Functions ---

/// Encrypts the content of a file using a password.
pub fn encrypt_file(plaintext: &[u8], password: &str) -> std::result::Result<EncryptedFile, Unspecified> {
    let rng = SystemRandom::new();
    let mut salt = [0u8; PBKDF2_SALT_LEN];
    rng.fill(&mut salt)?;

    // 1. Derive the key from the password and salt (PBKDF2)
    let key_bytes = pbkdf2_derive_key(password, &salt);
    let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, &key_bytes).unwrap();
    let key = LessSafeKey::new(unbound_key);

    // 2. Create a random nonce (IV)
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    // 3. Encrypt the data (AEAD)
    let mut buffer = plaintext.to_vec();
    
    // Encrypt and append the authentication tag
    key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut buffer)?;

    Ok(EncryptedFile {
        salt,
        nonce: nonce_bytes,
        ciphertext_with_tag: buffer,
    })
}

/// Decrypts an encrypted file using the password.
pub fn decrypt_file(encrypted_file: EncryptedFile, password: &str) -> std::result::Result<Vec<u8>, Unspecified> {
    let salt = encrypted_file.salt;
    let nonce_bytes = encrypted_file.nonce;
    let mut buffer = encrypted_file.ciphertext_with_tag;

    // 1. Derive the key (must use the same salt and KDF)
    let key_bytes = pbkdf2_derive_key(password, &salt);
    let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, &key_bytes).unwrap();
    let key = LessSafeKey::new(unbound_key);

    // 2. Create the nonce
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    // 3. Decrypt the data (AEAD)
    // Decrypt and verify the tag. Decryption failure is an `Unspecified` error.
    let decrypted_data = key.open_in_place(nonce, aead::Aad::empty(), &mut buffer)?;

    Ok(decrypted_data.to_vec())
}

// Utility function for PBKDF2 key derivation
fn pbkdf2_derive_key(password: &str, salt: &[u8]) -> Vec<u8> {
    let mut key_bytes = vec![0u8; CHACHA20_POLY1305.key_len()];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(PBKDF2_ITERATIONS).unwrap(),
        salt,
        password.as_bytes(),
        &mut key_bytes,
    );
    key_bytes
}
