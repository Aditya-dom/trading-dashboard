// src/bin/auth_helper.rs - Standalone Zerodha authentication helper
use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{self, Write};

pub struct AuthHelper {
    client: Client,
    api_key: String,
    api_secret: String,
}

impl AuthHelper {
    pub fn new(api_key: String, api_secret: String) -> Self {
        let client = Client::builder()
            .user_agent("ZerodhaAuthHelper/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            api_secret,
        }
    }

    /// Generate checksum using SHA256
    pub fn generate_checksum(&self, request_token: &str) -> String {
        let message = format!("{}{}{}", self.api_key, request_token, self.api_secret);
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Interactive authentication flow
    pub async fn interactive_auth(&self) -> Result<String> {
        println!("🚀 Zerodha KiteConnect Authentication Helper");
        println!("============================================");
        println!();
        println!("API Key: {}", self.api_key);
        println!();

        // Generate login URL
        let login_url = format!(
            "https://kite.zerodha.com/connect/login?v=3&api_key={}",
            self.api_key
        );
        println!("📋 STEP 1: Open this URL in your browser:");
        println!("{}", login_url);
        println!();

        println!("📋 STEP 2: Complete the login process");
        println!("   - Enter your Zerodha credentials");
        println!("   - Complete 2FA if enabled");
        println!("   - You'll be redirected to a URL with request_token");
        println!();

        print!("📋 STEP 3: Paste the request_token from the redirect URL: ");
        io::stdout().flush()?;

        let mut request_token = String::new();
        io::stdin().read_line(&mut request_token)?;
        let request_token = request_token.trim();

        if request_token.is_empty() {
            anyhow::bail!("Request token cannot be empty");
        }

        println!();
        println!("🔐 Generating access token...");

        // Generate access token
        let access_token = self.generate_session(request_token).await?;

        println!("✅ Authentication successful!");
        println!("🔑 Access Token: {}", access_token);
        println!();
        println!("📝 Copy this token to your config.toml file:");
        println!("   access_token = \"{}\"", access_token);
        println!();
        println!("⚠️  Security Notes:");
        println!("   - Keep this token secure");
        println!("   - Token expires daily - you'll need to regenerate it");
        println!("   - Don't commit this token to version control");

        Ok(access_token)
    }

    /// Generate session using request token
    async fn generate_session(&self, request_token: &str) -> Result<String> {
        let checksum = self.generate_checksum(request_token);
        let url = "https://api.kite.trade/session/token";

        let mut params = HashMap::new();
        params.insert("api_key", self.api_key.as_str());
        params.insert("request_token", request_token);
        params.insert("checksum", checksum.as_str());

        println!("   📡 API Key: {}", self.api_key);
        println!("   📡 Request Token: {}", request_token);
        println!("   📡 Checksum: {}", checksum);

        let response = self
            .client
            .post(url)
            .header("X-Kite-Version", "3")
            .form(&params)
            .send()
            .await?;

        let status_code = response.status();
        let response_text = response.text().await?;

        println!("   📥 Response Status: {}", status_code);

        if !status_code.is_success() {
            println!("   ❌ Response Body: {}", response_text);
            anyhow::bail!("Authentication failed with status: {}", status_code);
        }

        let json_response: Value = serde_json::from_str(&response_text)?;

        if json_response["status"] == "success" {
            let access_token = json_response["data"]["access_token"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Access token not found in response"))?;

            Ok(access_token.to_string())
        } else {
            let error_msg = json_response["message"].as_str().unwrap_or("Unknown error");
            anyhow::bail!("API Error: {}", error_msg);
        }
    }

    /// Test access token validity
    pub async fn test_token(&self, access_token: &str) -> Result<()> {
        println!("🧪 Testing access token validity...");

        let url = "https://api.kite.trade/user/profile";

        let response = self
            .client
            .get(url)
            .header("X-Kite-Version", "3")
            .header(
                "Authorization",
                format!("token {}:{}", self.api_key, access_token),
            )
            .send()
            .await?;

        let status_code = response.status();
        let response_text = response.text().await?;

        if status_code == 200 {
            let profile: Value = serde_json::from_str(&response_text)?;
            println!("✅ Token is valid!");
            println!(
                "   User: {}",
                profile["data"]["user_name"].as_str().unwrap_or("Unknown")
            );
            println!(
                "   Email: {}",
                profile["data"]["email"].as_str().unwrap_or("Unknown")
            );
            Ok(())
        } else if status_code == 403 {
            println!("❌ Token is invalid or expired (403 Forbidden)");
            anyhow::bail!("Invalid access token");
        } else {
            println!("⚠️  Unexpected response: {}", response_text);
            anyhow::bail!("Token test failed with status: {}", status_code);
        }
    }
}

// Standalone binary for authentication
#[tokio::main]
async fn main() -> Result<()> {
    // Read API credentials from environment or config file or prompt
    let api_key = std::env::var("ZERODHA_API_KEY")
        .or_else(|_| {
            // Try to read from config.toml
            if let Ok(config_str) = std::fs::read_to_string("config.toml") {
                if let Ok(config) = toml::from_str::<toml::Value>(&config_str) {
                    if let Some(api_key) = config
                        .get("zerodha")
                        .and_then(|z| z.get("api_key"))
                        .and_then(|k| k.as_str())
                    {
                        return Ok(api_key.to_string());
                    }
                }
            }
            Err(std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| {
            print!("Enter your Zerodha API Key: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        });

    let api_secret = std::env::var("ZERODHA_API_SECRET")
        .or_else(|_| {
            // Try to read from config.toml
            if let Ok(config_str) = std::fs::read_to_string("config.toml") {
                if let Ok(config) = toml::from_str::<toml::Value>(&config_str) {
                    if let Some(api_secret) = config
                        .get("zerodha")
                        .and_then(|z| z.get("api_secret"))
                        .and_then(|s| s.as_str())
                    {
                        return Ok(api_secret.to_string());
                    }
                }
            }
            Err(std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| {
            print!("Enter your Zerodha API Secret: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        });

    if api_key.is_empty() || api_secret.is_empty() {
        anyhow::bail!("API Key and Secret are required");
    }

    let auth_helper = AuthHelper::new(api_key, api_secret);

    // Check if we should test an existing token
    if let Ok(existing_token) = std::env::var("ZERODHA_ACCESS_TOKEN") {
        if !existing_token.is_empty() && existing_token != "your_access_token_here" {
            println!("🔍 Found existing access token, testing validity...");
            match auth_helper.test_token(&existing_token).await {
                Ok(_) => {
                    println!("✅ Existing token is valid, no need to re-authenticate");
                    return Ok(());
                }
                Err(_) => {
                    println!("❌ Existing token is invalid, starting authentication flow...");
                }
            }
        }
    }

    // Start interactive authentication
    let access_token = auth_helper.interactive_auth().await?;

    // Test the new token
    auth_helper.test_token(&access_token).await?;

    println!();
    println!("🎉 Authentication completed successfully!");
    println!("💡 Tips:");
    println!("   - Update your config.toml with the new access token");
    println!("   - Set environment variable to avoid re-entering credentials:");
    println!("     export ZERODHA_ACCESS_TOKEN=\"{}\"", access_token);
    println!("   - Restart your trading dashboard after updating the token");

    Ok(())
}
