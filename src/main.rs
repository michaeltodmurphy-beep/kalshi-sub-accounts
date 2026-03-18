use reqwest::header::{HeaderMap, HeaderValue};
use rsa::{
    pkcs8::DecodePrivateKey,
    pss::{Signature, SigningKey},
    sha2::Sha256,
    signature::RandomizedSigner,
    signature::SignatureEncoding,
    RsaPrivateKey,
};
use serde::Deserialize;
use std::io::{self, BufRead, Write};
use std::time::{SystemTime, UNIX_EPOCH};

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Config {
    kalshi_access_key: String,
}

// ── Models ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CreateSubaccountResponse {
    subaccount_number: u32,
}

// ── Kalshi Client ───────────────────────────────────────────────────────────

const BASE_URL: &str = "https://api.elections.kalshi.com/trade-api/v2";

struct KalshiClient {
    api_key: String,
    signing_key: SigningKey<Sha256>,
    http: reqwest::Client,
}

impl KalshiClient {
    /// Create a new client from an API key and a PEM-encoded RSA private key.
    fn new(api_key: &str, private_key_pem: &str) -> Self {
        let rsa_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)
            .expect("Failed to parse RSA private key PEM");
        let signing_key = SigningKey::<Sha256>::new(rsa_key);

        Self {
            api_key: api_key.to_string(),
            signing_key,
            http: reqwest::Client::new(),
        }
    }

    /// Build the authentication headers required by the Kalshi API.
    /// Signature = RSA-PSS-sign( timestamp_ms + method + path ), hex-encoded.
    fn auth_headers(&self, method: &str, path: &str) -> HeaderMap {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        let message = format!("{}{}{}", timestamp_ms, method, path);
        let mut rng = rand::thread_rng();
        let signature: Signature = self.signing_key.sign_with_rng(&mut rng, message.as_bytes());
        let sig_hex = hex::encode(signature.to_bytes());

        let mut headers = HeaderMap::new();
        headers.insert(
            "KALSHI-ACCESS-KEY",
            HeaderValue::from_str(&self.api_key).unwrap(),
        );
        headers.insert(
            "KALSHI-ACCESS-TIMESTAMP",
            HeaderValue::from_str(&timestamp_ms).unwrap(),
        );
        headers.insert(
            "KALSHI-ACCESS-SIGNATURE",
            HeaderValue::from_str(&sig_hex).unwrap(),
        );
        headers
    }

    /// Create a new subaccount (up to 32 per user).
    async fn create_subaccount(&self) -> anyhow::Result<CreateSubaccountResponse> {
        let path = "/portfolio/subaccounts";
        let url = format!("{}{}", BASE_URL, path);
        let headers = self.auth_headers("POST", path);

        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .send()
            .await?;

        let status = resp.status();
        if status == 201 {
            let body = resp.json::<CreateSubaccountResponse>().await?;
            Ok(body)
        } else {
            let text = resp.text().await.unwrap_or_else(|e| format!("Failed to read response body: {}", e));
            anyhow::bail!("HTTP {}: {}", status, text);
        }
    }
}

// ── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load API key from config.json
    let config_str = std::fs::read_to_string("config.json")
        .expect("Could not read config.json — please create it from config.json.example");
    let config: Config =
        serde_json::from_str(&config_str).expect("Failed to parse config.json");

    // Load RSA private key from kalshi-private-key.pem
    let private_key_pem = std::fs::read_to_string("kalshi-private-key.pem")
        .expect("Could not read kalshi-private-key.pem");

    let client = KalshiClient::new(&config.kalshi_access_key, &private_key_pem);

    println!("Kalshi Subaccount Manager");
    println!("Type 'Create Sub' to create a subaccount, 'exit' or 'quit' to quit.");

    let stdin = io::stdin();
    loop {
        print!("kalshi> ");
        io::stdout().flush()?;

        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            // EOF (e.g. Ctrl-D)
            break;
        }

        let cmd = line.trim().to_lowercase();
        match cmd.as_str() {
            "create sub" => {
                println!("Creating subaccount...");
                match client.create_subaccount().await {
                    Ok(resp) => {
                        println!(
                            "Subaccount created! subaccount_number = {}",
                            resp.subaccount_number
                        );
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
            "exit" | "quit" => {
                println!("Goodbye!");
                break;
            }
            "" => {}
            other => {
                println!("Unknown command: '{}'", other);
                println!("Available commands:");
                println!("  Create Sub  — create the next subaccount (up to 32)");
                println!("  exit / quit — exit the program");
            }
        }
    }

    Ok(())
}
