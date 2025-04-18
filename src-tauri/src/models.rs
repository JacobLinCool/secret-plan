use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents the breach status of a credential
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreachState {
    /// Not checked against breach database
    Unknown = 0,
    /// Checked and not found in breach database
    Safe = 1,
    /// Found in breach database
    Compromised = 2,
}

impl Default for BreachState {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Secret data that will be encrypted
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Secret {
    /// The password
    pub password: String,
    /// Additional notes
    pub notes: Option<String>,
    /// Time-based one-time password details
    pub totp: Option<String>,
    /// Additional custom fields (key-value pairs)
    pub custom_fields: HashMap<String, String>,
}

/// Represents a credential (login information)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// Unique identifier
    pub uuid: String,
    /// Website or application name
    pub site: String,
    /// Username or email
    pub username: String,
    /// Encrypted secret data as base64 string
    pub secret_enc: String,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Expiration timestamp (if any)
    pub expires_at: Option<DateTime<Utc>>,
    /// Password strength score (0-100)
    pub strength: u8,
    /// Breach status
    pub breach_state: BreachState,
}

impl Credential {
    pub fn new(site: String, username: String, secret_enc: String) -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4().to_string(),
            site,
            username,
            secret_enc,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            expires_at: None,
            strength: 0,
            breach_state: BreachState::Unknown,
        }
    }
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Argon2 memory cost
    pub argon2_memory_kb: u32,
    /// Argon2 iterations
    pub argon2_iterations: u32,
    /// Argon2 parallelism
    pub argon2_parallelism: u32,
    /// Whether to use biometrics for unlock when available
    pub use_biometrics: bool,
    /// Auto-lock timeout in minutes (0 = never)
    pub auto_lock_timeout: u32,
    /// Whether to sync with cloud storage
    pub enable_sync: bool,
    /// Sync provider details
    pub sync_config: Option<HashMap<String, String>>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            argon2_memory_kb: 65536, // 64 MB
            argon2_iterations: 3,
            argon2_parallelism: 4,
            use_biometrics: true,
            auto_lock_timeout: 5,
            enable_sync: false,
            sync_config: None,
        }
    }
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Entry ID
    pub id: i64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Action performed
    pub action: String,
    /// Item UUID if related to a credential
    pub item_uuid: Option<String>,
}
