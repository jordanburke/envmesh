// Cryptography module for encryption/decryption
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use argon2::password_hash::{rand_core::RngCore, SaltString};
use argon2::{Argon2, PasswordHasher};

pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    pub fn new(password: &str) -> Result<Self> {
        // Derive key from password using Argon2
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Password hashing failed: {}", e))?;

        // Extract the hash bytes for AES key
        let hash_bytes = password_hash.hash.unwrap();
        let key_bytes = &hash_bytes.as_bytes()[..32];

        let cipher = Aes256Gcm::new_from_slice(key_bytes)
            .map_err(|e| anyhow!("Key generation failed: {}", e))?;

        Ok(Self { cipher })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 12 {
            return Err(anyhow!("Invalid ciphertext: too short"));
        }

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt the data
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let crypto = Crypto::new("test_password").unwrap();
        let plaintext = b"Hello, World!";

        let encrypted = crypto.encrypt(plaintext).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }
}
