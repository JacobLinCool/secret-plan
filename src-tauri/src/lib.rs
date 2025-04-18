// Export modules
pub mod crypto;
pub mod error;
pub mod hibp;
pub mod models;
pub mod sqlite_repo;
pub mod strength;
#[cfg(test)]
pub mod tests;
pub mod traits;
pub mod vault;

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};

use hibp::HibpService;
use models::{AppSettings, BreachState, Credential, Secret};
use vault::{CredentialFilter, VaultManager};

// App state that will be shared across commands
pub struct AppState {
    vault_manager: Option<VaultManager>,
    hibp_service: HibpService,
    app_handle: Option<AppHandle>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vault_manager: None,
            hibp_service: HibpService::new(),
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }
}

// Helper function to get the vault database path
fn get_vault_path(app_handle: &AppHandle) -> PathBuf {
    let app_dir = app_handle.path().app_data_dir().unwrap();
    std::fs::create_dir_all(&app_dir).unwrap();
    app_dir.join("vault.db")
}

// ========== Tauri Commands ==========

#[tauri::command]
async fn initialize_vault(
    app_handle: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<bool, String> {
    let vault_path = get_vault_path(&app_handle);
    let vault_exists = vault_path.exists();
    let mut state_guard = state.lock().unwrap();
    if state_guard.vault_manager.is_none() {
        let settings = AppSettings::default();
        use crate::sqlite_repo::SqliteRepository;
        use crate::strength::SimpleStrengthCalculator;
        use std::sync::Arc;
        let repo = Arc::new(
            SqliteRepository::new(&vault_path).map_err(|e| format!("Failed to open DB: {}", e))?,
        );
        let strength = Arc::new(SimpleStrengthCalculator);
        let vault_manager =
            VaultManager::new(repo.clone(), repo.clone(), repo.clone(), strength, settings)
                .map_err(|e| format!("Failed to initialize vault: {}", e))?;
        state_guard.vault_manager = Some(vault_manager);
        state_guard.set_app_handle(app_handle);
    }
    Ok(vault_exists)
}

#[tauri::command]
async fn create_vault(
    master_password: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let mut state_guard = state.lock().unwrap();
    let vault_manager = state_guard
        .vault_manager
        .as_mut()
        .ok_or_else(|| "Vault not initialized".to_string())?;

    // Unlock (which will create a new vault if it doesn't exist)
    vault_manager
        .unlock(&master_password)
        .map_err(|e| format!("Failed to create vault: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn unlock_vault(
    master_password: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<bool, String> {
    let mut state_guard = state.lock().unwrap();
    let vault_manager = state_guard
        .vault_manager
        .as_mut()
        .ok_or_else(|| "Vault not initialized".to_string())?;

    // Attempt to unlock
    match vault_manager.unlock(&master_password) {
        Ok(_) => Ok(true),
        Err(error::AppError::AuthFailed) => Ok(false),
        Err(e) => Err(format!("Error unlocking vault: {}", e)),
    }
}

#[tauri::command]
async fn lock_vault(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let mut state_guard = state.lock().unwrap();
    let vault_manager = state_guard
        .vault_manager
        .as_mut()
        .ok_or_else(|| "Vault not initialized".to_string())?;

    vault_manager
        .lock()
        .map_err(|e| format!("Failed to lock vault: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn is_vault_locked(state: State<'_, Mutex<AppState>>) -> Result<bool, String> {
    let state_guard = state.lock().unwrap();
    let vault_manager = state_guard
        .vault_manager
        .as_ref()
        .ok_or_else(|| "Vault not initialized".to_string())?;

    Ok(!vault_manager.is_unlocked())
}

#[tauri::command]
async fn add_credential(
    site: String,
    username: String,
    password: String,
    notes: Option<String>,
    totp: Option<String>,
    custom_fields: Option<serde_json::Value>,
    state: State<'_, Mutex<AppState>>,
) -> Result<Credential, String> {
    let state_guard = state.lock().unwrap();
    let vault_manager = state_guard
        .vault_manager
        .as_ref()
        .ok_or_else(|| "Vault not initialized".to_string())?;

    // Parse custom fields if provided
    let custom_fields_map = match custom_fields {
        Some(value) => serde_json::from_value(value)
            .map_err(|e| format!("Invalid custom fields format: {}", e))?,
        None => std::collections::HashMap::new(),
    };

    // Create secret
    let secret = Secret {
        password,
        notes,
        totp,
        custom_fields: custom_fields_map,
    };

    // Add credential to vault
    let credential = vault_manager
        .add_credential(&site, &username, secret)
        .map_err(|e| format!("Failed to add credential: {}", e))?;

    Ok(credential)
}

#[tauri::command]
async fn get_credential(
    uuid: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<Credential, String> {
    let state_guard = state.lock().unwrap();
    let vault_manager = state_guard
        .vault_manager
        .as_ref()
        .ok_or_else(|| "Vault not initialized".to_string())?;

    let credential = vault_manager
        .get_credential(&uuid)
        .map_err(|e| format!("Failed to get credential: {}", e))?;

    Ok(credential)
}

#[tauri::command]
async fn get_credential_secret(
    uuid: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<Secret, String> {
    let state_guard = state.lock().unwrap();
    let vault_manager = state_guard
        .vault_manager
        .as_ref()
        .ok_or_else(|| "Vault not initialized".to_string())?;

    // Get the credential
    let credential = vault_manager
        .get_credential(&uuid)
        .map_err(|e| format!("Failed to get credential: {}", e))?;

    // Decrypt the secret
    let secret = vault_manager
        .decrypt_secret(&credential)
        .map_err(|e| format!("Failed to decrypt secret: {}", e))?;

    Ok(secret)
}

#[tauri::command]
async fn delete_credential(uuid: String, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let state_guard = state.lock().unwrap();
    let vault_manager = state_guard
        .vault_manager
        .as_ref()
        .ok_or_else(|| "Vault not initialized".to_string())?;

    vault_manager
        .delete_credential(&uuid)
        .map_err(|e| format!("Failed to delete credential: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn search_credentials(
    search_term: Option<String>,
    tag: Option<String>,
    min_strength: Option<u8>,
    breach_state: Option<i32>,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<Credential>, String> {
    let state_guard = state.lock().unwrap();
    let vault_manager = state_guard
        .vault_manager
        .as_ref()
        .ok_or_else(|| "Vault not initialized".to_string())?;

    // Convert breach_state from i32 to BreachState enum
    let breach_state_enum = match breach_state {
        Some(0) => Some(BreachState::Unknown),
        Some(1) => Some(BreachState::Safe),
        Some(2) => Some(BreachState::Compromised),
        Some(_) => None,
        None => None,
    };

    // Create filter
    let filter = CredentialFilter {
        search_term,
        tag,
        min_strength,
        breach_state: breach_state_enum,
    };

    // Get credentials
    let credentials = vault_manager
        .list_credentials(Some(filter))
        .map_err(|e| format!("Failed to search credentials: {}", e))?;

    Ok(credentials)
}

#[tauri::command]
async fn check_password_breach(
    uuid: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<BreachState, String> {
    // Extract only what is needed before await
    let (password, hibp_service);
    {
        let state_guard = state.lock().unwrap();
        let vault_manager = state_guard
            .vault_manager
            .as_ref()
            .ok_or_else(|| "Vault not initialized".to_string())?;
        let credential = vault_manager
            .get_credential(&uuid)
            .map_err(|e| format!("Failed to get credential: {}", e))?;
        let secret = vault_manager
            .decrypt_secret(&credential)
            .map_err(|e| format!("Failed to decrypt secret: {}", e))?;
        password = secret.password;
        hibp_service = state_guard.hibp_service.clone();
    }
    // Compute SHA-1 hash of the password
    let password_hash = hibp_service.compute_sha1_hash(password.as_bytes());
    // Check if the password is in the HIBP database
    let breach_state = hibp_service
        .check_password(&password_hash)
        .await
        .map_err(|e| format!("Failed to check password breach: {}", e))?;
    // Reacquire lock to update breach state
    {
        let mut state_guard = state.lock().unwrap();
        let vault_manager = state_guard
            .vault_manager
            .as_mut()
            .ok_or_else(|| "Vault not initialized".to_string())?;
        vault_manager
            .update_breach_state(&uuid, breach_state)
            .map_err(|e| format!("Failed to update breach state: {}", e))?;
    }
    Ok(breach_state)
}

#[tauri::command]
async fn get_app_settings(state: State<'_, Mutex<AppState>>) -> Result<AppSettings, String> {
    let state_guard = state.lock().unwrap();
    let vault_manager = state_guard
        .vault_manager
        .as_ref()
        .ok_or_else(|| "Vault not initialized".to_string())?;

    let settings = vault_manager
        .get_settings()
        .map_err(|e| format!("Failed to get app settings: {}", e))?;

    Ok(settings)
}

#[tauri::command]
async fn save_app_settings(
    settings: AppSettings,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let state_guard = state.lock().unwrap();
    let vault_manager = state_guard
        .vault_manager
        .as_ref()
        .ok_or_else(|| "Vault not initialized".to_string())?;

    vault_manager
        .save_settings(&settings)
        .map_err(|e| format!("Failed to save app settings: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn generate_password(
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_numbers: bool,
    use_symbols: bool,
    exclude_similar: bool,
) -> Result<String, String> {
    if length < 1 {
        return Err("Password length must be at least 1".to_string());
    }

    if !(use_uppercase || use_lowercase || use_numbers || use_symbols) {
        return Err("At least one character type must be selected".to_string());
    }

    // Define character sets
    let upper = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let lower = "abcdefghijklmnopqrstuvwxyz";
    let numbers = "0123456789";
    let symbols = "!@#$%^&*()-_=+[]{}|;:,.<>?";

    // Similar looking characters
    let similar = "Il1O0";

    // Build the charset
    let mut charset = String::new();

    if use_uppercase {
        charset.push_str(upper);
    }

    if use_lowercase {
        charset.push_str(lower);
    }

    if use_numbers {
        charset.push_str(numbers);
    }

    if use_symbols {
        charset.push_str(symbols);
    }

    // Remove similar characters if requested
    if exclude_similar {
        for c in similar.chars() {
            charset = charset.replace(c, "");
        }
    }

    // Generate the password
    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| {
            let idx = rand::Rng::gen_range(&mut rng, 0..charset.len());
            charset.chars().nth(idx).unwrap()
        })
        .collect();

    Ok(password)
}

#[tauri::command]
async fn greet(name: &str) -> Result<String, String> {
    Ok(format!("Hello, {}! You've been greeted from Rust!", name))
}

// Entrypoint
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create app state
    let app_state = Mutex::new(AppState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            initialize_vault,
            create_vault,
            unlock_vault,
            lock_vault,
            is_vault_locked,
            add_credential,
            get_credential,
            get_credential_secret,
            delete_credential,
            search_credentials,
            check_password_breach,
            get_app_settings,
            save_app_settings,
            generate_password,
        ])
        .setup(move |app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
