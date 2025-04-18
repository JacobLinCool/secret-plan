use sha1::{Digest, Sha1};

use crate::error::{AppError, AppResult};
use crate::models::BreachState;

/// Service to check passwords against the HIBP API
#[derive(Clone)]
pub struct HibpService {
    api_base_url: String,
    user_agent: String,
}

impl Default for HibpService {
    fn default() -> Self {
        Self::new()
    }
}

impl HibpService {
    /// Creates a new HibpService
    pub fn new() -> Self {
        Self {
            api_base_url: "https://api.pwnedpasswords.com".to_string(),
            user_agent: format!("SecretPlanApp/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    /// Checks if a password has been exposed in data breaches
    /// Uses the k-anonymity model: only the first 5 chars of the hash are sent to the API
    pub async fn check_password(&self, password_hash: &str) -> AppResult<BreachState> {
        if password_hash.len() != 40 {
            return Err(AppError::Other("Invalid SHA-1 hash length".to_string()));
        }

        // Split the hash for k-anonymity (first 5 chars and the rest)
        let prefix = &password_hash[0..5];
        let suffix = &password_hash[5..];

        // Build the request URL
        let url = format!("{}/range/{}", self.api_base_url, prefix);

        // Create HTTP client with a timeout
        let client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| AppError::Other(format!("Failed to create HTTP client: {}", e)))?;

        // Send the request
        let response = client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| AppError::Other(format!("Failed to send request to HIBP API: {}", e)))?;

        // Check if the request was successful
        if !response.status().is_success() {
            return Err(AppError::Other(format!(
                "HIBP API returned error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        // Get the response text
        let body = response
            .text()
            .await
            .map_err(|e| AppError::Other(format!("Failed to read HIBP API response: {}", e)))?;

        // Parse the response and check if our hash suffix is in the list
        self.check_hash_in_response(suffix, &body)
    }

    /// Computes the SHA-1 hash of the input data
    pub fn compute_sha1_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha1::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:X}", result) // Uppercase hex format
    }

    /// Parses the HIBP API response and checks if the hash suffix is in the list
    fn check_hash_in_response(
        &self,
        hash_suffix: &str,
        response_body: &str,
    ) -> AppResult<BreachState> {
        // The response is a list of hash suffixes and counts, separated by colons and new lines
        // Example: 0018A45C4D1DEF81644B54AB7F969B88D65:1

        for line in response_body.lines() {
            let parts: Vec<&str> = line.trim().split(':').collect();
            if parts.len() == 2 {
                let suffix = parts[0];
                let count: u64 = parts[1].parse().unwrap_or(0);

                // Case-insensitive comparison of hash suffixes
                if suffix.eq_ignore_ascii_case(hash_suffix) && count > 0 {
                    return Ok(BreachState::Compromised);
                }
            }
        }

        // If we got here, the hash wasn't found in the list
        Ok(BreachState::Safe)
    }
}
