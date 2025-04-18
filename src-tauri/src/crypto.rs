use aes_gcm::{
    aead::{Aead, KeyInit, OsRng, Payload},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::SaltString, Argon2, Params, PasswordHasher, PasswordVerifier, Version,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult, CryptoError};
use crate::models::AppSettings;

/// Encrypted container format (used for secrets in vault_items)
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
    /// Stored Argon2 hash of the master password for verification
    master_password_hash: Option<String>,
    /// Application settings for KDF parameters
    settings: AppSettings,
    settings_repo: Option<std::sync::Arc<dyn crate::traits::SettingsRepository>>, // Add repository reference
}

impl CryptoService {
    /// Creates a new CryptoService instance (locked state)
    pub fn new(settings: AppSettings) -> Self {
        Self {
            master_key: None,
            master_password_hash: None, // Will be loaded or created during unlock/init
            settings,
            settings_repo: None,
        }
    }

    pub fn with_settings_repo(
        mut self,
        repo: std::sync::Arc<dyn crate::traits::SettingsRepository>,
    ) -> Self {
        self.settings_repo = Some(repo);
        self
    }

    /// Derives a key AND generates a password hash from the master password using Argon2.
    /// This should only be called when *creating* a new vault or *changing* the master password.
    fn derive_key_and_hash(&self, master_password: &str) -> AppResult<(Key<Aes256Gcm>, String)> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = self.get_argon2_instance()?;

        // Generate a 32-byte (256-bit) key
        let mut key_bytes = [0u8; 32];
        argon2
            .hash_password_into(
                master_password.as_bytes(),
                salt.as_str().as_bytes(),
                &mut key_bytes,
            )
            .map_err(|e| CryptoError::KeyDerivation(format!("Key derivation failed: {}", e)))?;

        // Generate the password hash for storage and verification
        let password_hash = argon2
            .hash_password(master_password.as_bytes(), &salt)
            .map_err(|e| CryptoError::KeyDerivation(format!("Password hashing failed: {}", e)))?
            .to_string();

        Ok((*Key::<Aes256Gcm>::from_slice(&key_bytes), password_hash))
    }

    /// Verifies the master password against the stored hash and derives the key if successful.
    fn verify_password_and_derive_key(
        &self,
        master_password: &str,
        stored_hash: &str,
    ) -> AppResult<Key<Aes256Gcm>> {
        let argon2 = self.get_argon2_instance()?;
        let parsed_hash = argon2::PasswordHash::new(stored_hash).map_err(|e| {
            CryptoError::KeyDerivation(format!("Invalid stored hash format: {}", e))
        })?;

        // Verify the password
        argon2
            .verify_password(master_password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::AuthFailed)?; // Use AuthFailed for incorrect password

        // If verification succeeded, *re-derive* the key using the salt from the stored hash
        let salt = parsed_hash
            .salt
            .ok_or_else(|| CryptoError::KeyDerivation("Missing salt in stored hash".to_string()))?;
        let mut key_bytes = [0u8; 32];
        argon2
            .hash_password_into(
                master_password.as_bytes(),
                salt.as_str().as_bytes(),
                &mut key_bytes,
            )
            .map_err(|e| CryptoError::KeyDerivation(format!("Key re-derivation failed: {}", e)))?;

        Ok(*Key::<Aes256Gcm>::from_slice(&key_bytes))
    }

    /// Unlocks the CryptoService with the given master password.
    /// This now involves loading the stored hash (if available) and verifying.
    pub fn unlock(&mut self, master_password: &str) -> AppResult<()> {
        // Load the master_password_hash from the repository if available
        if let Some(repo) = &self.settings_repo {
            self.master_password_hash = repo.get_master_password_hash()?;
        }

        let (key, hash_to_store) = match &self.master_password_hash {
            Some(stored_hash) => {
                // Verify existing password and derive key
                let key = self.verify_password_and_derive_key(master_password, stored_hash)?;
                (key, stored_hash.clone()) // Keep the existing hash
            }
            None => {
                // First time unlock / vault creation: derive key and hash
                let (key, new_hash) = self.derive_key_and_hash(master_password)?;
                // Store the new hash in the repository
                if let Some(repo) = &self.settings_repo {
                    repo.save_master_password_hash(&new_hash)?;
                }
                (key, new_hash)
            }
        };

        self.master_key = Some(key);
        self.master_password_hash = Some(hash_to_store);
        Ok(())
    }

    /// Locks the CryptoService by removing the derived key
    pub fn lock(&mut self) {
        self.master_key = None;
        // Keep master_password_hash loaded
    }

    /// Checks if the CryptoService is unlocked
    pub fn is_unlocked(&self) -> bool {
        self.master_key.is_some()
    }

    /// Encrypts plaintext data using AES-256-GCM, returning JSON container.
    pub fn encrypt(&self, plaintext: &[u8], associated_data: &[u8]) -> AppResult<String> {
        let (nonce_bytes, ciphertext) = self.encrypt_raw(plaintext, associated_data)?;

        // Package the nonce and ciphertext in our container format
        let container = EncryptedContainer {
            nonce: BASE64.encode(nonce_bytes),
            ciphertext: BASE64.encode(ciphertext),
        };

        // Serialize and return the container
        serde_json::to_string(&container).map_err(|e| {
            CryptoError::Encryption(format!("Failed to serialize container: {}", e)).into()
        })
    }

    /// Encrypts plaintext data, returning raw nonce and ciphertext.
    /// Useful for settings where nonce needs to be stored separately.
    pub fn encrypt_return_nonce(
        &self,
        plaintext: &[u8],
        associated_data: &[u8],
    ) -> AppResult<(Vec<u8>, Vec<u8>)> {
        let (nonce_bytes, ciphertext) = self.encrypt_raw(plaintext, associated_data)?;
        Ok((nonce_bytes.to_vec(), ciphertext))
    }

    /// Core encryption logic.
    fn encrypt_raw(
        &self,
        plaintext: &[u8],
        associated_data: &[u8],
    ) -> AppResult<([u8; 12], Vec<u8>)> {
        let key = self.get_key()?;

        // Generate a random 96-bit (12-byte) nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher instance
        let cipher = Aes256Gcm::new(key);

        // Encrypt the plaintext with associated data
        let ciphertext = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: plaintext,
                    aad: associated_data,
                },
            )
            .map_err(|e| CryptoError::Encryption(format!("Encryption failed: {:?}", e)))?;

        Ok((nonce_bytes, ciphertext))
    }

    /// Decrypts ciphertext from a JSON container using AES-256-GCM.
    pub fn decrypt(&self, encrypted_container: &str, associated_data: &[u8]) -> AppResult<Vec<u8>> {
        // Parse the container
        let container: EncryptedContainer = serde_json::from_str(encrypted_container)
            .map_err(|e| CryptoError::InvalidFormat(format!("Invalid container format: {}", e)))?;

        // Decode base64 components
        let nonce_bytes = BASE64
            .decode(&container.nonce)
            .map_err(|e| CryptoError::InvalidFormat(format!("Invalid nonce encoding: {}", e)))?;

        let ciphertext = BASE64.decode(&container.ciphertext).map_err(|e| {
            CryptoError::InvalidFormat(format!("Invalid ciphertext encoding: {}", e))
        })?;

        // Decrypt using the raw decryption method
        self.decrypt_raw(&ciphertext, associated_data, &nonce_bytes)
    }

    /// Decrypts raw ciphertext using AES-256-GCM with a provided nonce.
    /// Useful for settings where nonce is stored separately.
    pub fn decrypt_with_nonce(
        &self,
        ciphertext: &[u8],
        associated_data: &[u8],
        nonce_bytes: &[u8],
    ) -> AppResult<Vec<u8>> {
        self.decrypt_raw(ciphertext, associated_data, nonce_bytes)
    }

    /// Core decryption logic.
    fn decrypt_raw(
        &self,
        ciphertext: &[u8],
        associated_data: &[u8],
        nonce_bytes: &[u8],
    ) -> AppResult<Vec<u8>> {
        let key = self.get_key()?;

        if nonce_bytes.len() != 12 {
            return Err(CryptoError::InvalidFormat("Nonce must be 12 bytes".to_string()).into());
        }
        let nonce = Nonce::from_slice(nonce_bytes);

        // Create cipher instance
        let cipher = Aes256Gcm::new(key);

        // Decrypt the ciphertext with associated data
        let plaintext = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: ciphertext,
                    aad: associated_data,
                },
            )
            .map_err(|e| CryptoError::Decryption(format!("Decryption failed: {:?}", e)))?;

        Ok(plaintext)
    }

    /// Helper to get the master key or return an error if locked
    fn get_key(&self) -> AppResult<&Key<Aes256Gcm>> {
        self.master_key.as_ref().ok_or(AppError::VaultLocked)
    }

    /// Updates the key derivation parameters
    pub fn update_kdf_settings(&mut self, settings: AppSettings) {
        // TODO: Consider if changing KDF settings should require re-hashing the master password
        self.settings = settings;
    }

    /// Helper to configure Argon2 instance based on settings
    fn get_argon2_instance(&self) -> AppResult<Argon2> {
        let params = Params::new(
            self.settings.argon2_memory_kb,
            self.settings.argon2_iterations,
            self.settings.argon2_parallelism,
            Some(32), // Output length for key derivation
        )
        .map_err(|e| {
            CryptoError::KeyDerivation(format!("Failed to build Argon2 parameters: {}", e))
        })?;

        Ok(Argon2::new(
            argon2::Algorithm::Argon2id,
            Version::V0x13,
            params,
        ))
    }
}
