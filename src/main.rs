use reqwest::header::{HeaderMap, HeaderValue};
use rsa::{
    pkcs8::DecodePrivateKey,
    pss::{SigningKey, Signature},
    sha2::Sha256,
    signature::RandomizedSigner,
    signature::SignatureEncoding,
    RsaPrivateKey,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// ── Models ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CreateSubaccountResponse {
    subaccount_number: u32,
}

#[derive(Debug, Deserialize)]
struct Subaccount {
    subaccount_number: u32,
}

#[derive(Debug, Deserialize)]
struct ListSubaccountsResponse {
    subaccounts: Vec<Subaccount>,
}

#[derive(Debug, Deserialize)]
struct Balance {
    balance: i64, // in cents
}

#[derive(Debug, Serialize)]
struct TransferRequest {
    from_subaccount: u32,
    to_subaccount: u32,
    amount: i64, // in cents
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
    /// Signature = RSA-PSS-sign( timestamp + method + path )
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

    // ── Subaccount Operations ───────────────────────────────────────────

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
            .await?
            .error_for_status()?
            .json::<CreateSubaccountResponse>()
            .await?;

        Ok(resp)
    }

    /// List all subaccounts.
    async fn list_subaccounts(&self) -> anyhow::Result<ListSubaccountsResponse> {
        let path = "/portfolio/subaccounts";
        let url = format!("{}{}", BASE_URL, path);
        let headers = self.auth_headers("GET", path);

        let resp = self
            .http
            .get(&url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?
            .json::<ListSubaccountsResponse>()
            .await?;

        Ok(resp)
    }

    /// Get balance for a specific subaccount.
    async fn get_subaccount_balance(&self, subaccount_number: u32) -> anyhow::Result<Balance> {
        let path = format!("/portfolio/subaccounts/{}/balance", subaccount_number);
        let url = format!("{}{}", BASE_URL, path);
        let headers = self.auth_headers("GET", &path);

        let resp = self
            .http
            .get(&url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?
            .json::<Balance>()
            .await?;

        Ok(resp)
    }

    /// Transfer funds between two subaccounts.
    async fn transfer_between_subaccounts(
        &self,
        from: u32,
        to: u32,
        amount_cents: i64,
    ) -> anyhow::Result<()> {
        let path = "/portfolio/subaccounts/transfer";
        let url = format!("{}{}", BASE_URL, path);
        let headers = self.auth_headers("POST", path);

        let body = TransferRequest {
            from_subaccount: from,
            to_subaccount: to,
            amount: amount_cents,
        };

        self.http
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

// ── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load credentials from environment variables
    let api_key = std::env::var("KALSHI_API_KEY").expect("Set KALSHI_API_KEY env var");
    let private_key_pem = std::fs::read_to_string(
        std::env::var("KALSHI_PRIVATE_KEY_PATH")
            .unwrap_or_else(|_| "kalshi_private_key.pem".to_string()),
    )
    .expect("Could not read private key PEM file");

    let client = KalshiClient::new(&api_key, &private_key_pem);

    // 1. Create a subaccount
    println!("Creating subaccount…");
    let created = client.create_subaccount().await?;
    println!("  → subaccount_number = {}", created.subaccount_number);

    // 2. List all subaccounts
    println!("\nListing subaccounts…");
    let list = client.list_subaccounts().await?;
    for sa in &list.subaccounts {
        println!("  • subaccount #{}", sa.subaccount_number);
    }

    // 3. Check balance on the new subaccount
    println!(
        "\nGetting balance for subaccount #{}…",
        created.subaccount_number
    );
    let balance = client
        .get_subaccount_balance(created.subaccount_number)
        .await?;
    println!("  → balance = {} cents", balance.balance);

    // 4. Transfer $10.00 (1000 cents) from subaccount 0 (main) to the new one
    println!(
        "\nTransferring 1000 cents from main → subaccount #{}…",
        created.subaccount_number
    );
    client
        .transfer_between_subaccounts(0, created.subaccount_number, 1000)
        .await?;
    println!("  → transfer complete");

    Ok(())
}
