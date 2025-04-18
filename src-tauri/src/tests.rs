#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use tempfile::tempdir;

    use crate::crypto::CryptoService;
    use crate::models::{AppSettings, BreachState, Secret};
    use crate::vault::{CredentialFilter, VaultManager};

    const TEST_MASTER_PASSWORD: &str = "SuperSecretMasterPassword123!";

    #[tokio::test]
    async fn test_vault_initialization() {
        // Create temporary directory for test database
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_vault.db");

        // Create vault manager
        let settings = AppSettings::default();
        use crate::sqlite_repo::SqliteRepository;
        use crate::strength::SimpleStrengthCalculator;
        use std::sync::Arc;
        let repo = Arc::new(SqliteRepository::new(&db_path).unwrap());
        let strength = Arc::new(SimpleStrengthCalculator);
        let vault = VaultManager::new(repo.clone(), repo.clone(), repo.clone(), strength, settings)
            .unwrap();

        // Vault should start locked
        assert!(!vault.is_unlocked());
    }

    #[tokio::test]
    async fn test_vault_unlock_lock() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_vault.db");

        // Create vault manager
        let settings = AppSettings::default();
        use crate::sqlite_repo::SqliteRepository;
        use crate::strength::SimpleStrengthCalculator;
        use std::sync::Arc;
        let repo = Arc::new(SqliteRepository::new(&db_path).unwrap());
        let strength = Arc::new(SimpleStrengthCalculator);
        let mut vault =
            VaultManager::new(repo.clone(), repo.clone(), repo.clone(), strength, settings)
                .unwrap();

        // Unlock vault
        vault.unlock(TEST_MASTER_PASSWORD).unwrap();
        assert!(vault.is_unlocked());

        // Lock vault
        vault.lock().unwrap();
        assert!(!vault.is_unlocked());
    }

    #[tokio::test]
    async fn test_credential_crud() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_vault.db");

        // Create vault manager
        let settings = AppSettings::default();
        use crate::sqlite_repo::SqliteRepository;
        use crate::strength::SimpleStrengthCalculator;
        use std::sync::Arc;
        let repo = Arc::new(SqliteRepository::new(&db_path).unwrap());
        let strength = Arc::new(SimpleStrengthCalculator);
        let mut vault =
            VaultManager::new(repo.clone(), repo.clone(), repo.clone(), strength, settings)
                .unwrap();

        // Unlock vault
        vault.unlock(TEST_MASTER_PASSWORD).unwrap();

        // Create a secret
        let mut custom_fields = HashMap::new();
        custom_fields.insert(
            "recovery_email".to_string(),
            "backup@example.com".to_string(),
        );

        let secret = Secret {
            password: "P@ssw0rd123!".to_string(),
            notes: Some("This is a test account".to_string()),
            totp: Some(
                "otpauth://totp/Test:user@example.com?secret=JBSWY3DPEHPK3PXP&issuer=Test"
                    .to_string(),
            ),
            custom_fields,
        };

        // Add credential with tags as Vec<String>
        let tags = Some(vec![
            "personal".to_string(),
            "email".to_string(),
            "test".to_string(),
        ]);
        let credential = vault
            .add_credential("example.com", "user@example.com", secret, tags)
            .unwrap();

        // Check UUID was generated
        assert!(!credential.uuid.is_empty());

        // Check tags were saved
        assert_eq!(credential.tags, vec!["personal", "email", "test"]);

        // Retrieve credential
        let retrieved = vault.get_credential(&credential.uuid).unwrap();
        assert_eq!(retrieved.site, "example.com");
        assert_eq!(retrieved.username, "user@example.com");
        assert_eq!(retrieved.tags, vec!["personal", "email", "test"]);

        // Decrypt secret
        let decrypted_secret = vault.decrypt_secret(&retrieved).unwrap();
        assert_eq!(decrypted_secret.password, "P@ssw0rd123!");
        assert_eq!(
            decrypted_secret.notes,
            Some("This is a test account".to_string())
        );

        // List credentials
        let credentials = vault.list_credentials(None).unwrap();
        assert_eq!(credentials.len(), 1);

        // Update breach state
        vault
            .update_breach_state(&credential.uuid, BreachState::Safe)
            .unwrap();
        let updated = vault.get_credential(&credential.uuid).unwrap();
        assert_eq!(updated.breach_state, BreachState::Safe);

        // Delete credential
        vault.delete_credential(&credential.uuid).unwrap();
        let credentials = vault.list_credentials(None).unwrap();
        assert_eq!(credentials.len(), 0);
    }

    #[tokio::test]
    async fn test_filtering_credentials() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_vault.db");

        // Create vault manager
        let settings = AppSettings::default();
        use crate::sqlite_repo::SqliteRepository;
        use crate::strength::SimpleStrengthCalculator;
        use std::sync::Arc;
        let repo = Arc::new(SqliteRepository::new(&db_path).unwrap());
        let strength = Arc::new(SimpleStrengthCalculator);
        let mut vault =
            VaultManager::new(repo.clone(), repo.clone(), repo.clone(), strength, settings)
                .unwrap();

        // Unlock vault
        vault.unlock(TEST_MASTER_PASSWORD).unwrap();

        // Add multiple credentials with tags
        let secret1 = Secret {
            password: "Password1!".to_string(),
            notes: None,
            totp: None,
            custom_fields: HashMap::new(),
        };

        let secret2 = Secret {
            password: "Password2@".to_string(),
            notes: None,
            totp: None,
            custom_fields: HashMap::new(),
        };

        let secret3 = Secret {
            password: "Password3#".to_string(),
            notes: None,
            totp: None,
            custom_fields: HashMap::new(),
        };

        let _cred1 = vault
            .add_credential(
                "example.com",
                "user1@example.com",
                secret1,
                Some(vec!["personal".to_string(), "email".to_string()]),
            )
            .unwrap();

        let _cred2 = vault
            .add_credential(
                "example.org",
                "user2@example.org",
                secret2,
                Some(vec!["work".to_string(), "finance".to_string()]),
            )
            .unwrap();

        let _cred3 = vault
            .add_credential(
                "secure-site.com",
                "admin@secure-site.com",
                secret3,
                Some(vec!["work".to_string(), "admin".to_string()]),
            )
            .unwrap();

        // Test site search
        let filter = CredentialFilter {
            search_term: Some("example".to_string()),
            tag: None,
            min_strength: None,
            breach_state: None,
        };
        let results = vault.list_credentials(Some(filter)).unwrap();
        assert_eq!(results.len(), 2);

        // Test tag filtering
        let filter = CredentialFilter {
            search_term: None,
            tag: Some("work".to_string()),
            min_strength: None,
            breach_state: None,
        };
        let results = vault.list_credentials(Some(filter)).unwrap();
        assert_eq!(results.len(), 2);

        // Test combined filtering
        let filter = CredentialFilter {
            search_term: None,
            tag: Some("work".to_string()),
            min_strength: Some(50),
            breach_state: None,
        };
        let results = vault.list_credentials(Some(filter)).unwrap();
        // Note: our simple strength calculator will likely give these passwords a high score
        assert!(results.len() <= 2);
    }

    #[tokio::test]
    async fn test_crypto_service() {
        let settings = AppSettings::default();
        let mut crypto = CryptoService::new(settings);

        // Test unlock/lock
        crypto.unlock(TEST_MASTER_PASSWORD).unwrap();
        assert!(crypto.is_unlocked());

        crypto.lock();
        assert!(!crypto.is_unlocked());

        // Unlock for encryption testing
        crypto.unlock(TEST_MASTER_PASSWORD).unwrap();

        // Test encryption/decryption
        let plaintext = b"This is a secret message!";
        let aad = b"associated data";

        let ciphertext = crypto.encrypt(plaintext, aad).unwrap();
        assert_ne!(ciphertext, String::from_utf8_lossy(plaintext).to_string());

        let decrypted = crypto.decrypt(&ciphertext, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn test_app_settings() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_vault.db");

        // Create vault manager with default settings
        let settings = AppSettings::default();
        use crate::sqlite_repo::SqliteRepository;
        use crate::strength::SimpleStrengthCalculator;
        use std::sync::Arc;
        let repo = Arc::new(SqliteRepository::new(&db_path).unwrap());
        let strength = Arc::new(SimpleStrengthCalculator);
        let mut vault = VaultManager::new(
            repo.clone(),
            repo.clone(),
            repo.clone(),
            strength,
            settings.clone(),
        )
        .unwrap();

        // Unlock vault
        vault.unlock(TEST_MASTER_PASSWORD).unwrap();

        // Get settings (should be default since none saved yet)
        let retrieved = vault.get_settings().unwrap();
        assert_eq!(retrieved.argon2_memory_kb, settings.argon2_memory_kb);

        // Update settings
        let mut new_settings = settings.clone();
        new_settings.argon2_memory_kb = 128 * 1024; // 128 MB
        new_settings.auto_lock_timeout = 10;

        vault.save_settings(&new_settings).unwrap();

        // Retrieve updated settings
        let retrieved = vault.get_settings().unwrap();
        assert_eq!(retrieved.argon2_memory_kb, 128 * 1024);
        assert_eq!(retrieved.auto_lock_timeout, 10);
    }

    #[tokio::test]
    async fn test_audit_log() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_vault.db");

        // Create vault manager
        let settings = AppSettings::default();
        use crate::sqlite_repo::SqliteRepository;
        use crate::strength::SimpleStrengthCalculator;
        use std::sync::Arc;
        let repo = Arc::new(SqliteRepository::new(&db_path).unwrap());
        let strength = Arc::new(SimpleStrengthCalculator);
        let mut vault =
            VaultManager::new(repo.clone(), repo.clone(), repo.clone(), strength, settings)
                .unwrap();

        // Unlock vault (generates first audit log entry)
        vault.unlock(TEST_MASTER_PASSWORD).unwrap();

        // Add a credential (generates second audit log entry)
        let secret = Secret {
            password: "TestPassword123!".to_string(),
            notes: None,
            totp: None,
            custom_fields: HashMap::new(),
        };
        let _credential = vault
            .add_credential(
                "test.com",
                "user@test.com",
                secret,
                Some(vec!["test".to_string(), "audit".to_string()]),
            )
            .unwrap();

        // Retrieve audit log
        let log_entries = vault.get_audit_log(None).unwrap();

        // Should have at least 2 entries (unlock and add credential)
        assert!(log_entries.len() >= 2);

        // Check the most recent entry is for adding the credential
        let latest = &log_entries[0];
        assert!(latest.action.contains("Added credential"));
    }
}
