use crate::error::AppResult;
use crate::models::{AuditLogEntry, BreachState, Credential};
use crate::vault::CredentialFilter;

// Trait for managing credentials
// Needs Send + Sync bounds if used with Arc<Mutex<dyn ...>> across threads
pub trait CredentialRepository: Send + Sync {
    fn add_credential(&self, credential: &Credential, strength: u8) -> AppResult<()>;
    fn update_credential(&self, credential: &Credential) -> AppResult<()>;
    // Returns site name for audit log upon successful deletion
    fn delete_credential(&self, uuid: &str) -> AppResult<String>;
    fn get_credential(&self, uuid: &str) -> AppResult<Credential>;
    fn list_credentials(&self, filter: Option<CredentialFilter>) -> AppResult<Vec<Credential>>;
    fn update_breach_state(&self, uuid: &str, state: BreachState) -> AppResult<()>;
    fn credential_exists(&self, uuid: &str) -> AppResult<bool>;
}

// Trait for managing application settings
pub trait SettingsRepository: Send + Sync {
    // Returns Option<(nonce, encrypted_settings)> or None if not found
    fn get_encrypted_settings(&self) -> AppResult<Option<(Vec<u8>, Vec<u8>)>>;
    // Saves nonce and encrypted settings blob
    fn save_encrypted_settings(&self, nonce: &[u8], encrypted_settings: &[u8]) -> AppResult<()>;
    fn get_master_password_hash(&self) -> AppResult<Option<String>>;
    fn save_master_password_hash(&self, hash: &str) -> AppResult<()>;
}

// Trait for logging audit events
pub trait AuditLogger: Send + Sync {
    fn add_log(&self, action: &str, item_uuid: Option<&str>) -> AppResult<i64>;
    fn get_logs(&self, limit: Option<i64>) -> AppResult<Vec<AuditLogEntry>>;
}

// Trait for calculating password strength
pub trait PasswordStrengthCalculator: Send + Sync {
    fn calculate_strength(&self, password: &str) -> u8;
}
