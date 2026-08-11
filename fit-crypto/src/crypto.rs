use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use fit_core::{FitError, FitResult};
use rand::RngCore;

pub struct CryptoEngine;

impl CryptoEngine {
    pub fn derive_key(password: &str, salt: &[u8; 16]) -> FitResult<[u8; 32]> {
        let params = Params::new(16384, 2, 1, Some(32))
            .map_err(|e| FitError::Crypto(format!("Argon2 params error: {}", e)))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut derived_key = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut derived_key)
            .map_err(|e| FitError::Crypto(format!("Argon2 KDF failed: {}", e)))?;

        Ok(derived_key)
    }

    pub fn generate_salt() -> [u8; 16] {
        let mut salt = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);
        salt
    }

    pub fn generate_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        nonce
    }

    pub fn encrypt_payload(data: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> FitResult<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(key.into());
        let nonce_obj = Nonce::from_slice(nonce);
        cipher
            .encrypt(nonce_obj, data)
            .map_err(|e| FitError::Crypto(format!("Encryption failed: {}", e)))
    }

    pub fn decrypt_payload(encrypted_data: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> FitResult<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(key.into());
        let nonce_obj = Nonce::from_slice(nonce);
        cipher
            .decrypt(nonce_obj, encrypted_data)
            .map_err(|_| FitError::InvalidPassword)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_roundtrip() {
        let password = "SuperSecretPassword123!";
        let salt = CryptoEngine::generate_salt();
        let nonce = CryptoEngine::generate_nonce();
        let key = CryptoEngine::derive_key(password, &salt).unwrap();

        let data = b"Confidential FIT payload data";
        let encrypted = CryptoEngine::encrypt_payload(data, &key, &nonce).unwrap();
        assert_ne!(data.to_vec(), encrypted);

        let decrypted = CryptoEngine::decrypt_payload(&encrypted, &key, &nonce).unwrap();
        assert_eq!(data.to_vec(), decrypted);
    }
}
