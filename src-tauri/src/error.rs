use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Cryptography error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Vault is locked")]
    VaultLocked,

    #[error("Authentication failed")]
    AuthFailed,

    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Key derivation error: {0}")]
    KeyDerivation(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Random generation error: {0}")]
    Random(String),
}

pub type AppResult<T> = Result<T, AppError>;
