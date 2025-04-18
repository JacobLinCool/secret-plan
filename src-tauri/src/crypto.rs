use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{password_hash::SaltString, Argon2, Params};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::{AppResult, CryptoError};
use crate::models::AppSettings;

/// Encrypted container format
#[derive(Debug, Serialize, Deserialize)]
struct EncryptedContainer {
    /// Base64-encoded nonce
    nonce: String,
    /// Base64-encoded ciphertext
    ciphertext: String,
}

/// Handles all cryptographic operations
pub struct CryptoService {
    /// Encryption key derived from master password
    master_key: Option<Key<Aes256Gcm>>,
    /// Application settings for KDF parameters
    settings: AppSettings,
}

impl CryptoService {
    /// Creates a new CryptoService instance (locked state)
    pub fn new(settings: AppSettings) -> Self {
        Self {
            master_key: None,
            settings,
        }
    }

    /// Derives a key from the master password using Argon2
    pub fn derive_key(&self, master_password: &str) -> AppResult<Key<Aes256Gcm>> {
        let salt = SaltString::generate(&mut OsRng);

        // Configure Argon2 parameters based on app settings
        let params = Params::new(
            self.settings.argon2_memory_kb,
            self.settings.argon2_iterations,
            self.settings.argon2_parallelism,
            Some(32), // Output length: 32 bytes (256 bits)
        )
        .map_err(|e| {
            CryptoError::KeyDerivation(format!("Failed to build Argon2 parameters: {}", e))
        })?;

        // Use Argon2id (balanced between Argon2i and Argon2d)
        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

        // Generate a 32-byte (256-bit) key
        let mut key_bytes = [0u8; 32];
        argon2
            .hash_password_into(
                master_password.as_bytes(),
                salt.as_str().as_bytes(),
                &mut key_bytes,
            )
            .map_err(|e| CryptoError::KeyDerivation(format!("Key derivation failed: {}", e)))?;

        Ok(*Key::<Aes256Gcm>::from_slice(&key_bytes))
    }

    /// Unlocks the CryptoService with the given master password
    pub fn unlock(&mut self, master_password: &str) -> AppResult<()> {
        self.master_key = Some(self.derive_key(master_password)?);
        Ok(())
    }

    /// Locks the CryptoService by removing the derived key
    pub fn lock(&mut self) {
        self.master_key = None;
    }

    /// Checks if the CryptoService is unlocked
    pub fn is_unlocked(&self) -> bool {
        self.master_key.is_some()
    }

    /// Encrypts plaintext data using AES-256-GCM
    pub fn encrypt(&self, plaintext: &[u8], _associated_data: &[u8]) -> AppResult<String> {
        let key = self.get_key()?;

        // Generate a random 96-bit (12-byte) nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher instance
        let cipher = Aes256Gcm::new(key);

        // Encrypt the plaintext with associated data
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| CryptoError::Encryption(format!("Encryption failed: {:?}", e)))?;

        // Package the nonce and ciphertext in our container format
        let container = EncryptedContainer {
            nonce: BASE64.encode(nonce),
            ciphertext: BASE64.encode(ciphertext),
        };

        // Serialize and return the container
        let container_json = serde_json::to_string(&container).map_err(|e| {
            CryptoError::Encryption(format!("Failed to serialize container: {}", e))
        })?;

        Ok(container_json)
    }

    /// Decrypts ciphertext using AES-256-GCM
    pub fn decrypt(
        &self,
        encrypted_container: &str,
        _associated_data: &[u8],
    ) -> AppResult<Vec<u8>> {
        let key = self.get_key()?;

        // Parse the container
        let container: EncryptedContainer = serde_json::from_str(encrypted_container)
            .map_err(|e| CryptoError::InvalidFormat(format!("Invalid container format: {}", e)))?;

        // Decode base64 components
        let nonce_bytes = BASE64
            .decode(&container.nonce)
            .map_err(|e| CryptoError::InvalidFormat(format!("Invalid nonce encoding: {}", e)))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = BASE64.decode(&container.ciphertext).map_err(|e| {
            CryptoError::InvalidFormat(format!("Invalid ciphertext encoding: {}", e))
        })?;

        // Create cipher instance
        let cipher = Aes256Gcm::new(key);

        // Decrypt the ciphertext with associated data
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| CryptoError::Decryption(format!("Decryption failed: {:?}", e)))?;

        Ok(plaintext)
    }

    /// Helper to get the master key or return an error if locked
    fn get_key(&self) -> AppResult<&Key<Aes256Gcm>> {
        self.master_key
            .as_ref()
            .ok_or_else(|| crate::error::AppError::VaultLocked)
    }

    /// Updates the key derivation parameters
    pub fn update_kdf_settings(&mut self, settings: AppSettings) {
        self.settings = settings;
    }
}
