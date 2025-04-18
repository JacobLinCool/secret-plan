use crate::error::{AppError, AppResult};
use crate::models::{AuditLogEntry, BreachState, Credential};
use crate::traits::{AuditLogger, CredentialRepository, SettingsRepository};
use crate::vault::CredentialFilter; // Keep filter definition accessible
use chrono::{TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json;
use std::path::Path;
use std::sync::Mutex;

/// Concrete implementation for database operations using SQLite.
pub struct SqliteRepository {
    conn: Mutex<Connection>,
}

impl SqliteRepository {
    /// Creates a new repository and initializes the schema if needed.
    pub fn new(db_path: &Path) -> AppResult<Self> {
        let conn = Connection::open(db_path)?;
        Self::init_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Initializes the database schema.
    fn init_schema(conn: &Connection) -> AppResult<()> {
        conn.execute_batch(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value BLOB NOT NULL,
                nonce BLOB -- Added nonce for settings encryption
            );
            CREATE TABLE IF NOT EXISTS vault_items (
                uuid TEXT PRIMARY KEY,
                site TEXT NOT NULL,
                username TEXT NOT NULL,
                secret_enc TEXT NOT NULL, -- Storing encrypted secret as text (JSON container)
                tags TEXT NOT NULL DEFAULT '[]', -- Storing tags as JSON array
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                expires_at INTEGER,
                strength INTEGER NOT NULL DEFAULT 0,
                breach_state INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                action TEXT NOT NULL,
                item_uuid TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_vault_site ON vault_items(site);
            CREATE INDEX IF NOT EXISTS idx_vault_username ON vault_items(username);
            CREATE INDEX IF NOT EXISTS idx_vault_tags ON vault_items(tags);
            CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp);
            COMMIT;",
        )?;
        Ok(())
    }

    /// Helper to add an audit log entry within a transaction.
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
}

impl CredentialRepository for SqliteRepository {
    fn add_credential(&self, credential: &Credential, strength: u8) -> AppResult<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        // Serialize tags to JSON string
        let tags_json =
            serde_json::to_string(&credential.tags).map_err(|e| AppError::Serialization(e))?;

        tx.execute(
            "INSERT INTO vault_items (
                uuid, site, username, secret_enc, tags, created_at, updated_at, expires_at, strength, breach_state
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                credential.uuid,
                credential.site,
                credential.username,
                credential.secret_enc,
                tags_json,
                credential.created_at.timestamp(),
                credential.updated_at.timestamp(),
                credential.expires_at.map(|dt| dt.timestamp()),
                strength,
                credential.breach_state as i32,
            ],
        )?;

        self.add_audit_log_tx(
            &tx,
            &format!("Added credential for {}", credential.site),
            Some(&credential.uuid),
        )?;

        tx.commit()?;
        Ok(())
    }

    fn update_credential(&self, credential: &Credential) -> AppResult<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        if !self.credential_exists_tx(&tx, &credential.uuid)? {
            return Err(AppError::NotFound(credential.uuid.clone()));
        }

        let updated_at = Utc::now();
        tx.execute(
            "UPDATE vault_items SET 
                site = ?, username = ?, secret_enc = ?, tags = ?, updated_at = ?, expires_at = ?, strength = ?, breach_state = ? 
             WHERE uuid = ?",
            params![
                credential.site,
                credential.username,
                credential.secret_enc,
                serde_json::to_string(&credential.tags)?,
                updated_at.timestamp(),
                credential.expires_at.map(|dt| dt.timestamp()),
                credential.strength, // Assuming strength is recalculated and passed in Credential
                credential.breach_state as i32,
                credential.uuid,
            ],
        )?;

        self.add_audit_log_tx(
            &tx,
            &format!("Updated credential for {}", credential.site),
            Some(&credential.uuid),
        )?;

        tx.commit()?;
        Ok(())
    }

    fn delete_credential(&self, uuid: &str) -> AppResult<String> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let site: String = tx
            .query_row(
                "SELECT site FROM vault_items WHERE uuid = ?",
                params![uuid],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(uuid.to_string()),
                _ => AppError::Database(e),
            })?;

        tx.execute("DELETE FROM vault_items WHERE uuid = ?", params![uuid])?;

        self.add_audit_log_tx(&tx, &format!("Deleted credential for {}", site), Some(uuid))?;

        tx.commit()?;
        Ok(site) // Return site name for audit log message construction elsewhere
    }

    fn get_credential(&self, uuid: &str) -> AppResult<Credential> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT uuid, site, username, secret_enc, tags, created_at, updated_at, expires_at, strength, breach_state FROM vault_items WHERE uuid = ?",
            params![uuid],
            |row| {
                let created_ts: i64 = row.get(5)?;
                let updated_ts: i64 = row.get(6)?;
                let expires_ts: Option<i64> = row.get(7)?;
                let breach_state_int: i32 = row.get(9)?;
                let tags_json: String = row.get(4)?;

                // Deserialize tags from JSON string
                let tags = serde_json::from_str(&tags_json)
                    .map_err(|_e| rusqlite::Error::InvalidColumnType(4, "tags".to_string(), rusqlite::types::Type::Text))?;

                Ok(Credential {
                    uuid: row.get(0)?,
                    site: row.get(1)?,
                    username: row.get(2)?,
                    secret_enc: row.get(3)?,
                    tags,
                    created_at: Utc.timestamp_opt(created_ts, 0).single().ok_or(rusqlite::Error::InvalidColumnType(5, "created_at".to_string(), rusqlite::types::Type::Integer))?,
                    updated_at: Utc.timestamp_opt(updated_ts, 0).single().ok_or(rusqlite::Error::InvalidColumnType(6, "updated_at".to_string(), rusqlite::types::Type::Integer))?,
                    expires_at: expires_ts.and_then(|ts| Utc.timestamp_opt(ts, 0).single()),
                    strength: row.get(8)?,
                    breach_state: match breach_state_int {
                        1 => BreachState::Safe,
                        2 => BreachState::Compromised,
                        _ => BreachState::Unknown,
                    },
                })
            },
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(uuid.to_string()),
            _ => AppError::Database(e),
        })
    }

    fn list_credentials(&self, filter: Option<CredentialFilter>) -> AppResult<Vec<Credential>> {
        let conn = self.conn.lock().unwrap();
        let mut query = String::from(
            "SELECT uuid, site, username, secret_enc, tags, created_at, updated_at, expires_at, strength, breach_state FROM vault_items",
        );
        let mut conditions = Vec::new();
        let mut params_dyn: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(f) = filter {
            if let Some(term) = f.search_term {
                conditions.push("(site LIKE ?1 OR username LIKE ?1 OR tags LIKE ?1)".to_string());
                params_dyn.push(Box::new(format!("%{}%", term)));
            }
            if let Some(tag) = f.tag {
                // Use JSON_ARRAY_LENGTH to ensure it's an array first, then check if it contains tag
                // For SQLite 3.38.0+ you could use JSON_CONTAINS, but we use LIKE for compatibility
                conditions.push("(JSON_ARRAY_LENGTH(tags) > 0 AND tags LIKE ?)".to_string());
                params_dyn.push(Box::new(format!("%\"{}\"%", tag)));
            }
            if let Some(strength) = f.min_strength {
                conditions.push("strength >= ?".to_string());
                params_dyn.push(Box::new(strength));
            }
            if let Some(state) = f.breach_state {
                conditions.push("breach_state = ?".to_string());
                params_dyn.push(Box::new(state as i32));
            }
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        query.push_str(" ORDER BY site, username");

        let mut stmt = conn.prepare(&query)?;
        let params_ref: Vec<&dyn rusqlite::ToSql> = params_dyn.iter().map(|b| b.as_ref()).collect();

        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            let created_ts: i64 = row.get(5)?;
            let updated_ts: i64 = row.get(6)?;
            let expires_ts: Option<i64> = row.get(7)?;
            let breach_state_int: i32 = row.get(9)?;
            let tags_json: String = row.get(4)?;

            // Deserialize tags from JSON string
            let tags = serde_json::from_str(&tags_json).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    4,
                    "tags".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;

            Ok(Credential {
                uuid: row.get(0)?,
                site: row.get(1)?,
                username: row.get(2)?,
                secret_enc: row.get(3)?,
                tags,
                created_at: Utc.timestamp_opt(created_ts, 0).single().ok_or(
                    rusqlite::Error::InvalidColumnType(
                        5,
                        "created_at".to_string(),
                        rusqlite::types::Type::Integer,
                    ),
                )?,
                updated_at: Utc.timestamp_opt(updated_ts, 0).single().ok_or(
                    rusqlite::Error::InvalidColumnType(
                        6,
                        "updated_at".to_string(),
                        rusqlite::types::Type::Integer,
                    ),
                )?,
                expires_at: expires_ts.and_then(|ts| Utc.timestamp_opt(ts, 0).single()),
                strength: row.get(8)?,
                breach_state: match breach_state_int {
                    1 => BreachState::Safe,
                    2 => BreachState::Compromised,
                    _ => BreachState::Unknown,
                },
            })
        })?;

        let mut credentials = Vec::new();
        for row_result in rows {
            credentials.push(row_result?);
        }

        Ok(credentials)
    }

    fn update_breach_state(&self, uuid: &str, state: BreachState) -> AppResult<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let rows_affected = tx.execute(
            "UPDATE vault_items SET breach_state = ? WHERE uuid = ?",
            params![state as i32, uuid],
        )?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(uuid.to_string()));
        }

        let action = match state {
            BreachState::Safe => "Marked credential as safe",
            BreachState::Compromised => "Marked credential as compromised",
            BreachState::Unknown => "Reset credential breach state to unknown",
        };
        self.add_audit_log_tx(&tx, action, Some(uuid))?;

        tx.commit()?;
        Ok(())
    }

    fn credential_exists(&self, uuid: &str) -> AppResult<bool> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let exists = self.credential_exists_tx(&tx, uuid)?;
        tx.commit()?;
        Ok(exists)
    }
}

// Separate helper for transaction context
impl SqliteRepository {
    fn credential_exists_tx(&self, tx: &Transaction, uuid: &str) -> AppResult<bool> {
        let count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM vault_items WHERE uuid = ?",
            params![uuid],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}

impl SettingsRepository for SqliteRepository {
    fn get_encrypted_settings(&self) -> AppResult<Option<(Vec<u8>, Vec<u8>)>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT nonce, value FROM meta WHERE key = 'settings'",
            [],
            |row| {
                let nonce: Vec<u8> = row.get(0)?;
                let value: Vec<u8> = row.get(1)?;
                Ok((nonce, value))
            },
        )
        .optional()
        .map_err(AppError::Database)
    }

    fn save_encrypted_settings(&self, nonce: &[u8], encrypted_settings: &[u8]) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO meta (key, nonce, value) VALUES ('settings', ?, ?)",
            params![nonce, encrypted_settings],
        )?;
        Ok(())
    }

    fn get_master_password_hash(&self) -> AppResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT value FROM meta WHERE key = 'master_password_hash'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(AppError::Database)
    }

    fn save_master_password_hash(&self, hash: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('master_password_hash', ?)",
            [hash],
        )?;
        Ok(())
    }
}

impl AuditLogger for SqliteRepository {
    fn add_log(&self, action: &str, item_uuid: Option<&str>) -> AppResult<i64> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT INTO audit_log (timestamp, action, item_uuid) VALUES (?, ?, ?)",
            params![now, action, item_uuid],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn get_logs(&self, limit: Option<i64>) -> AppResult<Vec<AuditLogEntry>> {
        let conn = self.conn.lock().unwrap();
        let limit = limit.unwrap_or(100);
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, action, item_uuid FROM audit_log ORDER BY timestamp DESC LIMIT ?",
        )?;

        let rows = stmt.query_map([limit], |row| {
            let timestamp_val: i64 = row.get(1)?;
            Ok(AuditLogEntry {
                id: row.get(0)?,
                timestamp: Utc.timestamp_opt(timestamp_val, 0).single().ok_or(
                    rusqlite::Error::InvalidColumnType(
                        1,
                        "timestamp".to_string(),
                        rusqlite::types::Type::Integer,
                    ),
                )?,
                action: row.get(2)?,
                item_uuid: row.get(3)?,
            })
        })?;

        let mut entries = Vec::new();
        for row_result in rows {
            entries.push(row_result?);
        }
        Ok(entries)
    }
}
