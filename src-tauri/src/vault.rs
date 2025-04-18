use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::crypto::CryptoService;
use crate::error::{AppError, AppResult};
use crate::models::{AppSettings, AuditLogEntry, BreachState, Credential, Secret};

/// Filter options for listing credentials
pub struct CredentialFilter {
    pub search_term: Option<String>,
    pub tag: Option<String>,
    pub min_strength: Option<u8>,
    pub breach_state: Option<BreachState>,
}

/// Singleton manager for vault operations
pub struct VaultManager {
    /// Database connection wrapped in Mutex for thread safety
    conn: Mutex<Connection>,
    /// Crypto service for encryption/decryption
    crypto: Arc<Mutex<CryptoService>>,
    /// Whether the vault is currently unlocked
    is_unlocked: bool,
}

impl VaultManager {
    /// Creates a new VaultManager with the given database path
    pub fn new(db_path: &Path, settings: AppSettings) -> AppResult<Self> {
        // Open or create the database
        let conn = Connection::open(db_path)?;

        // Create database schema if it doesn't exist
        Self::init_schema(&conn)?;

        // Create the crypto service
        let crypto = Arc::new(Mutex::new(CryptoService::new(settings)));

        Ok(Self {
            conn: Mutex::new(conn),
            crypto,
            is_unlocked: false,
        })
    }

    /// Initializes the database schema
    fn init_schema(conn: &Connection) -> AppResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS meta (
                id TEXT PRIMARY KEY,
                value BLOB NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS vault_items (
                uuid TEXT PRIMARY KEY,
                site TEXT NOT NULL,
                username TEXT NOT NULL,
                secret_enc TEXT NOT NULL,
                tags TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                expires_at INTEGER,
                strength INTEGER NOT NULL,
                breach_state INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                action TEXT NOT NULL,
                item_uuid TEXT
            )",
            [],
        )?;

        // Create indexes for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vault_site ON vault_items(site)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vault_username ON vault_items(username)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp)",
            [],
        )?;

        Ok(())
    }

    /// Unlocks the vault with the master password
    pub fn unlock(&mut self, master_password: &str) -> AppResult<()> {
        // Unlock the crypto service
        let mut crypto = self.crypto.lock().unwrap();
        crypto.unlock(master_password)?;
        self.is_unlocked = true;

        // Log the unlock action
        self.add_audit_log("Vault unlocked", None)?;

        Ok(())
    }

    /// Locks the vault
    pub fn lock(&mut self) -> AppResult<()> {
        if self.is_unlocked {
            // Lock the crypto service
            let mut crypto = self.crypto.lock().unwrap();
            crypto.lock();
            self.is_unlocked = false;

            // Log the lock action
            self.add_audit_log("Vault locked", None)?;
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
            format!("{}:{}", site, username).as_bytes(),
        )?;

        // Create a new credential
        let credential = Credential::new(site.to_string(), username.to_string(), secret_enc);

        // Calculate password strength (simplified for now)
        let strength = self.calculate_password_strength(&secret.password);

        // Begin transaction
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        // Insert the credential
        tx.execute(
            "INSERT INTO vault_items (
                uuid, site, username, secret_enc, tags, 
                created_at, updated_at, expires_at, strength, breach_state
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                credential.uuid,
                credential.site,
                credential.username,
                credential.secret_enc,
                credential.tags,
                credential.created_at.timestamp(),
                credential.updated_at.timestamp(),
                credential.expires_at.map(|dt| dt.timestamp()),
                strength,
                BreachState::Unknown as i32,
            ],
        )?;

        // Add audit log entry
        self.add_audit_log_tx(
            &tx,
            &format!("Added credential for {}", site),
            Some(&credential.uuid),
        )?;

        // Commit transaction
        tx.commit()?;

        Ok(credential)
    }

    /// Updates an existing credential
    pub fn update_credential(&self, uuid: &str, updates: Credential) -> AppResult<()> {
        self.ensure_unlocked()?;

        // Begin transaction
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        // Check if credential exists
        if !self.credential_exists_tx(&tx, uuid)? {
            return Err(AppError::NotFound(format!(
                "Credential with UUID {} not found",
                uuid
            )));
        }

        // Update the credential
        tx.execute(
            "UPDATE vault_items SET 
                site = ?, username = ?, secret_enc = ?, tags = ?,
                updated_at = ?, expires_at = ?, strength = ?, breach_state = ?
            WHERE uuid = ?",
            params![
                updates.site,
                updates.username,
                updates.secret_enc,
                updates.tags,
                Utc::now().timestamp(),
                updates.expires_at.map(|dt| dt.timestamp()),
                updates.strength,
                updates.breach_state as i32,
                uuid,
            ],
        )?;

        // Add audit log entry
        self.add_audit_log_tx(
            &tx,
            &format!("Updated credential for {}", updates.site),
            Some(uuid),
        )?;

        // Commit transaction
        tx.commit()?;

        Ok(())
    }

    /// Deletes a credential by UUID
    pub fn delete_credential(&self, uuid: &str) -> AppResult<()> {
        self.ensure_unlocked()?;

        // Begin transaction
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        // Get site name for audit log
        let site: String = tx
            .query_row(
                "SELECT site FROM vault_items WHERE uuid = ?",
                params![uuid],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| {
                AppError::NotFound(format!("Credential with UUID {} not found", uuid))
            })?;

        // Delete the credential
        tx.execute("DELETE FROM vault_items WHERE uuid = ?", params![uuid])?;

        // Add audit log entry
        self.add_audit_log_tx(&tx, &format!("Deleted credential for {}", site), Some(uuid))?;

        // Commit transaction
        tx.commit()?;

        Ok(())
    }

    /// Gets a credential by UUID
    pub fn get_credential(&self, uuid: &str) -> AppResult<Credential> {
        self.ensure_unlocked()?;

        let conn = self.conn.lock().unwrap();
        let row = conn.query_row(
            "SELECT 
                uuid, site, username, secret_enc, tags, 
                created_at, updated_at, expires_at, strength, breach_state
            FROM vault_items 
            WHERE uuid = ?",
            params![uuid],
            |row| {
                let created_ts: i64 = row.get(5)?;
                let updated_ts: i64 = row.get(6)?;
                let expires_ts: Option<i64> = row.get(7)?;

                Ok(Credential {
                    uuid: row.get(0)?,
                    site: row.get(1)?,
                    username: row.get(2)?,
                    secret_enc: row.get(3)?,
                    tags: row.get(4)?,
                    created_at: chrono::DateTime::from_timestamp(created_ts, 0)
                        .unwrap_or_else(Utc::now),
                    updated_at: chrono::DateTime::from_timestamp(updated_ts, 0)
                        .unwrap_or_else(Utc::now),
                    expires_at: expires_ts
                        .map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)),
                    strength: row.get(8)?,
                    breach_state: match row.get::<_, i32>(9)? {
                        0 => BreachState::Unknown,
                        1 => BreachState::Safe,
                        2 => BreachState::Compromised,
                        _ => BreachState::Unknown,
                    },
                })
            },
        )?;

        Ok(row)
    }

    /// Lists credentials matching the filter criteria
    pub fn list_credentials(&self, filter: Option<CredentialFilter>) -> AppResult<Vec<Credential>> {
        self.ensure_unlocked()?;

        // Build the query based on filter
        let mut query = String::from(
            "SELECT 
                uuid, site, username, secret_enc, tags, 
                created_at, updated_at, expires_at, strength, breach_state
            FROM vault_items",
        );

        let mut conditions = Vec::new();
        let mut params = Vec::new();

        if let Some(filter) = filter {
            // Add search term condition
            if let Some(search_term) = filter.search_term {
                conditions.push("(site LIKE ? OR username LIKE ?)");
                let like_term = format!("%{}%", search_term);
                params.push(like_term.clone());
                params.push(like_term);
            }

            // Add tag condition
            if let Some(tag) = filter.tag {
                conditions.push("tags LIKE ?");
                params.push(format!("%{}%", tag));
            }

            // Add strength condition
            if let Some(min_strength) = filter.min_strength {
                conditions.push("strength >= ?");
                params.push(min_strength.to_string());
            }

            // Add breach state condition
            if let Some(breach_state) = filter.breach_state {
                conditions.push("breach_state = ?");
                params.push((breach_state as i32).to_string());
            }
        }

        // Append conditions to the query
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        // Execute query
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&query)?;
        let params_slice: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

        let rows = stmt.query_map(params_slice.as_slice(), |row| {
            let created_ts: i64 = row.get(5)?;
            let updated_ts: i64 = row.get(6)?;
            let expires_ts: Option<i64> = row.get(7)?;

            Ok(Credential {
                uuid: row.get(0)?,
                site: row.get(1)?,
                username: row.get(2)?,
                secret_enc: row.get(3)?,
                tags: row.get(4)?,
                created_at: chrono::DateTime::from_timestamp(created_ts, 0)
                    .unwrap_or_else(Utc::now),
                updated_at: chrono::DateTime::from_timestamp(updated_ts, 0)
                    .unwrap_or_else(Utc::now),
                expires_at: expires_ts
                    .map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)),
                strength: row.get(8)?,
                breach_state: match row.get::<_, i32>(9)? {
                    0 => BreachState::Unknown,
                    1 => BreachState::Safe,
                    2 => BreachState::Compromised,
                    _ => BreachState::Unknown,
                },
            })
        })?;

        let mut credentials = Vec::new();
        for row in rows {
            credentials.push(row?);
        }

        Ok(credentials)
    }

    /// Decrypts the secret data from a credential
    pub fn decrypt_secret(&self, credential: &Credential) -> AppResult<Secret> {
        self.ensure_unlocked()?;

        let crypto = self.crypto.lock().unwrap();
        let plaintext = crypto.decrypt(
            &credential.secret_enc,
            format!("{}:{}", credential.site, credential.username).as_bytes(),
        )?;

        let secret: Secret = serde_json::from_slice(&plaintext).map_err(AppError::Serialization)?;

        Ok(secret)
    }

    /// Updates the breach state for a credential
    pub fn update_breach_state(&self, uuid: &str, state: BreachState) -> AppResult<()> {
        self.ensure_unlocked()?;

        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        // Update breach state
        tx.execute(
            "UPDATE vault_items SET breach_state = ? WHERE uuid = ?",
            params![state as i32, uuid],
        )?;

        // Add audit log entry
        let action = match state {
            BreachState::Safe => "Marked credential as safe",
            BreachState::Compromised => "Marked credential as compromised",
            BreachState::Unknown => "Reset credential breach state",
        };

        self.add_audit_log_tx(&tx, action, Some(uuid))?;

        tx.commit()?;

        Ok(())
    }

    /// Gets the app settings
    pub fn get_settings(&self) -> AppResult<AppSettings> {
        let conn = self.conn.lock().unwrap();
        let settings_json: Option<Vec<u8>> = conn
            .query_row("SELECT value FROM meta WHERE id = 'settings'", [], |row| {
                row.get(0)
            })
            .optional()?;

        if let Some(settings_json) = settings_json {
            if self.is_unlocked {
                // If vault is unlocked, decrypt settings
                let crypto = self.crypto.lock().unwrap();
                let plaintext =
                    crypto.decrypt(&String::from_utf8_lossy(&settings_json), b"app_settings")?;

                let settings: AppSettings =
                    serde_json::from_slice(&plaintext).map_err(AppError::Serialization)?;

                Ok(settings)
            } else {
                // Return default settings if locked
                Ok(AppSettings::default())
            }
        } else {
            // No settings saved yet, return defaults
            Ok(AppSettings::default())
        }
    }

    /// Saves app settings
    pub fn save_settings(&self, settings: &AppSettings) -> AppResult<()> {
        self.ensure_unlocked()?;

        let settings_json = serde_json::to_vec(settings).map_err(AppError::Serialization)?;

        // First encrypt the settings
        let crypto = self.crypto.lock().unwrap();
        let encrypted = crypto.encrypt(&settings_json, b"app_settings")?;
        drop(crypto); // Explicitly drop the crypto lock

        // Then save to database
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO meta (id, value) VALUES (?, ?)",
            params!["settings", encrypted.as_bytes()],
        )?;
        drop(conn); // Explicitly drop the connection lock

        // Finally update crypto service settings
        let mut crypto = self.crypto.lock().unwrap();
        crypto.update_kdf_settings(settings.clone());

        self.add_audit_log("Updated app settings", None)?;

        Ok(())
    }

    /// Gets audit log entries
    pub fn get_audit_log(&self, limit: Option<i64>) -> AppResult<Vec<AuditLogEntry>> {
        self.ensure_unlocked()?;

        let limit = limit.unwrap_or(100);
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, timestamp, action, item_uuid 
            FROM audit_log 
            ORDER BY timestamp DESC 
            LIMIT ?",
        )?;

        let rows = stmt.query_map([limit], |row| {
            let timestamp: i64 = row.get(1)?;

            Ok(AuditLogEntry {
                id: row.get(0)?,
                timestamp: chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now),
                action: row.get(2)?,
                item_uuid: row.get(3)?,
            })
        })?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }

        Ok(entries)
    }

    /// Helper method to ensure the vault is unlocked
    fn ensure_unlocked(&self) -> AppResult<()> {
        if !self.is_unlocked {
            return Err(AppError::VaultLocked);
        }
        Ok(())
    }

    /// Helper method to check if a credential exists in the transaction
    fn credential_exists_tx(&self, tx: &Transaction, uuid: &str) -> AppResult<bool> {
        let count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM vault_items WHERE uuid = ?",
            params![uuid],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    /// Helper method to add an audit log entry within a transaction
    fn add_audit_log_tx(
        &self,
        tx: &Transaction,
        action: &str,
        item_uuid: Option<&str>,
    ) -> AppResult<i64> {
        let now = Utc::now().timestamp();

        tx.execute(
            "INSERT INTO audit_log (timestamp, action, item_uuid) VALUES (?, ?, ?)",
            params![now, action, item_uuid],
        )?;

        Ok(tx.last_insert_rowid())
    }

    /// Helper method to add an audit log entry
    fn add_audit_log(&self, action: &str, item_uuid: Option<&str>) -> AppResult<i64> {
        let now = Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO audit_log (timestamp, action, item_uuid) VALUES (?, ?, ?)",
            params![now, action, item_uuid],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Calculates password strength score (0-100)
    fn calculate_password_strength(&self, password: &str) -> u8 {
        // This is a simplified strength calculator
        // In a real implementation, you'd use zxcvbn or another library

        let length = password.len();
        let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        let mut score = 0;

        // Length score (up to 40 points)
        score += std::cmp::min(length * 2, 40) as u8;

        // Character variety (up to 60 points)
        if has_lowercase {
            score += 10;
        }
        if has_uppercase {
            score += 15;
        }
        if has_digit {
            score += 15;
        }
        if has_special {
            score += 20;
        }

        score
    }
}
