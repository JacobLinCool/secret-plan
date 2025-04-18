use chrono::Utc;
use serde_json;
use std::sync::{Arc, Mutex};

use crate::crypto::CryptoService;
use crate::error::{AppError, AppResult};
use crate::models::{AppSettings, AuditLogEntry, BreachState, Credential, Secret};
use crate::traits::{
    AuditLogger, CredentialRepository, PasswordStrengthCalculator, SettingsRepository,
};

/// Filter options for listing credentials
// Keep this struct here as it relates to the VaultManager's public API
#[derive(Default)] // Add default for easier construction
pub struct CredentialFilter {
    pub search_term: Option<String>,
    pub tag: Option<String>,
    pub min_strength: Option<u8>,
    pub breach_state: Option<BreachState>,
}

/// Singleton manager for vault operations, orchestrating dependencies.
pub struct VaultManager {
    // Dependencies injected via traits
    credential_repo: Arc<dyn CredentialRepository>,
    settings_repo: Arc<dyn SettingsRepository>,
    audit_logger: Arc<dyn AuditLogger>,
    strength_calculator: Arc<dyn PasswordStrengthCalculator>,
    crypto: Arc<Mutex<CryptoService>>,
    is_unlocked: bool,
}

impl VaultManager {
    /// Creates a new VaultManager with injected dependencies.
    pub fn new(
        credential_repo: Arc<dyn CredentialRepository>,
        settings_repo: Arc<dyn SettingsRepository>,
        audit_logger: Arc<dyn AuditLogger>,
        strength_calculator: Arc<dyn PasswordStrengthCalculator>,
        settings: AppSettings, // Initial settings for CryptoService
    ) -> AppResult<Self> {
        // Create the crypto service and inject the settings repository
        let crypto = Arc::new(Mutex::new(
            CryptoService::new(settings).with_settings_repo(settings_repo.clone()),
        ));
        Ok(Self {
            credential_repo,
            settings_repo,
            audit_logger,
            strength_calculator,
            crypto,
            is_unlocked: false,
        })
    }

    /// Unlocks the vault with the master password
    pub fn unlock(&mut self, master_password: &str) -> AppResult<()> {
        // Unlock the crypto service
        let mut crypto = self.crypto.lock().unwrap();
        crypto.unlock(master_password)?;
        self.is_unlocked = true;
        drop(crypto); // Release lock before logging

        // Log the unlock action via the AuditLogger trait
        self.audit_logger.add_log("Vault unlocked", None)?;

        Ok(())
    }

    /// Locks the vault
    pub fn lock(&mut self) -> AppResult<()> {
        if self.is_unlocked {
            // Lock the crypto service
            let mut crypto = self.crypto.lock().unwrap();
            crypto.lock();
            self.is_unlocked = false;
            drop(crypto); // Release lock before logging

            // Log the lock action via the AuditLogger trait
            self.audit_logger.add_log("Vault locked", None)?;
        }
        Ok(())
    }

    /// Checks if the vault is unlocked
    pub fn is_unlocked(&self) -> bool {
        self.is_unlocked
    }

    /// Adds a new credential to the vault
    pub fn add_credential(
        &self,
        site: &str,
        username: &str,
        secret: Secret,
    ) -> AppResult<Credential> {
        self.ensure_unlocked()?;

        // Encrypt the secret
        let secret_json = serde_json::to_string(&secret).map_err(AppError::Serialization)?;
        let crypto = self.crypto.lock().unwrap();
        let secret_enc = crypto.encrypt(
            secret_json.as_bytes(),
            format!("{}:{}", site, username).as_bytes(), // AAD
        )?;
        drop(crypto);

        // Create a new credential struct
        let mut credential = Credential::new(site.to_string(), username.to_string(), secret_enc);

        // Calculate password strength using the injected calculator
        let strength = self
            .strength_calculator
            .calculate_strength(&secret.password);
        credential.strength = strength;

        // Persist using the CredentialRepository trait
        self.credential_repo.add_credential(&credential, strength)?;

        Ok(credential)
    }

    /// Updates an existing credential
    pub fn update_credential(
        &self,
        uuid: &str,
        site: &str,
        username: &str,
        secret: Secret,
        tags: String,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> AppResult<()> {
        self.ensure_unlocked()?;

        // Encrypt the updated secret
        let secret_json = serde_json::to_string(&secret).map_err(AppError::Serialization)?;
        let crypto = self.crypto.lock().unwrap();
        let secret_enc = crypto.encrypt(
            secret_json.as_bytes(),
            format!("{}:{}", site, username).as_bytes(), // AAD
        )?;
        drop(crypto);

        // Calculate new strength
        let strength = self
            .strength_calculator
            .calculate_strength(&secret.password);

        // Fetch existing to preserve created_at and potentially breach_state
        // Alternatively, the update method in the repo could handle partial updates
        let mut existing_credential = self.credential_repo.get_credential(uuid)?;

        // Update fields
        existing_credential.site = site.to_string();
        existing_credential.username = username.to_string();
        existing_credential.secret_enc = secret_enc;
        existing_credential.tags = tags;
        existing_credential.updated_at = Utc::now();
        existing_credential.expires_at = expires_at;
        existing_credential.strength = strength;
        // Keep existing_credential.breach_state
        // Keep existing_credential.created_at

        // Persist changes using the CredentialRepository trait
        self.credential_repo
            .update_credential(&existing_credential)?;

        // Audit log handled by repository's update_credential

        Ok(())
    }

    /// Deletes a credential by UUID
    pub fn delete_credential(&self, uuid: &str) -> AppResult<()> {
        self.ensure_unlocked()?;

        // Delete using the CredentialRepository trait
        // The repository handles the audit log internally
        self.credential_repo.delete_credential(uuid)?;
        Ok(())
    }

    /// Gets a credential by UUID (metadata only, no decrypted secret)
    pub fn get_credential(&self, uuid: &str) -> AppResult<Credential> {
        self.ensure_unlocked()?;
        self.credential_repo.get_credential(uuid)
    }

    /// Lists credentials matching the filter criteria
    pub fn list_credentials(&self, filter: Option<CredentialFilter>) -> AppResult<Vec<Credential>> {
        self.ensure_unlocked()?;
        self.credential_repo.list_credentials(filter)
    }

    /// Decrypts the secret data from a credential
    pub fn decrypt_secret(&self, credential: &Credential) -> AppResult<Secret> {
        self.ensure_unlocked()?;

        let crypto = self.crypto.lock().unwrap();
        let plaintext = crypto.decrypt(
            &credential.secret_enc,
            format!("{}:{}", credential.site, credential.username).as_bytes(), // AAD
        )?;

        let secret: Secret = serde_json::from_slice(&plaintext).map_err(AppError::Serialization)?;
        Ok(secret)
    }

    /// Updates the breach state for a credential
    pub fn update_breach_state(&self, uuid: &str, state: BreachState) -> AppResult<()> {
        self.ensure_unlocked()?;
        // Update using the CredentialRepository trait
        // The repository handles the audit log internally
        self.credential_repo.update_breach_state(uuid, state)
    }

    /// Gets the app settings, decrypting them first.
    pub fn get_settings(&self) -> AppResult<AppSettings> {
        self.ensure_unlocked()?;

        // Fetch encrypted settings and nonce from the SettingsRepository
        let encrypted_data = self.settings_repo.get_encrypted_settings()?;

        if let Some((nonce, encrypted_settings)) = encrypted_data {
            // Decrypt using CryptoService
            let crypto = self.crypto.lock().unwrap();
            let settings_json =
                crypto.decrypt_with_nonce(&encrypted_settings, b"app_settings", &nonce)?;
            drop(crypto);

            // Deserialize
            serde_json::from_slice(&settings_json).map_err(AppError::Serialization)
        } else {
            // If no settings saved yet, return default
            Ok(AppSettings::default())
        }
    }

    /// Saves app settings, encrypting them first.
    pub fn save_settings(&self, settings: &AppSettings) -> AppResult<()> {
        self.ensure_unlocked()?;

        let settings_json = serde_json::to_vec(settings).map_err(AppError::Serialization)?;

        // Encrypt the settings using CryptoService
        let crypto = self.crypto.lock().unwrap();
        // Use a method that returns nonce and ciphertext separately
        let (nonce, encrypted_settings) =
            crypto.encrypt_return_nonce(&settings_json, b"app_settings")?;
        drop(crypto); // Explicitly drop the crypto lock

        // Save nonce and encrypted data using SettingsRepository
        self.settings_repo
            .save_encrypted_settings(&nonce, &encrypted_settings)?;

        // Update crypto service's in-memory settings (KDF params)
        let mut crypto = self.crypto.lock().unwrap();
        crypto.update_kdf_settings(settings.clone());
        drop(crypto);

        // Log the action using AuditLogger
        self.audit_logger.add_log("Updated app settings", None)?;

        Ok(())
    }

    /// Gets audit log entries
    pub fn get_audit_log(&self, limit: Option<i64>) -> AppResult<Vec<AuditLogEntry>> {
        self.ensure_unlocked()?;
        // Fetch logs using the AuditLogger trait
        self.audit_logger.get_logs(limit)
    }

    /// Helper method to ensure the vault is unlocked
    fn ensure_unlocked(&self) -> AppResult<()> {
        if !self.is_unlocked {
            Err(AppError::VaultLocked)
        } else {
            Ok(())
        }
    }
}
