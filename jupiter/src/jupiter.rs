use reqwest::Client;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use solana_sdk::transaction::VersionedTransaction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signer::{Signer, keypair::Keypair};
use solana_sdk::commitment_config::CommitmentConfig;
use base64::Engine;
use std::env;
 // You might need to add `base64` to Cargo.toml imports if using newer version
pub struct JupiterClient {
    pub http: Client,
    pub base_url: String,
    pub api_key: Option<String>,
}

impl JupiterClient {
    pub fn new() -> Self {
        dotenv::dotenv().ok();

        let http = Client::new();
        let base_url = "https://quote-api.jup.ag/v3".to_string();
        let api_key = std::env::var("JUPITER_API_KEY").ok();

        Self { http, base_url, api_key }
    }


    pub async fn get_price(&self, token: &str) -> Result<f64, anyhow::Error> {
        let url = format!("https://api.jup.ag/price/v3?ids={}", token);
        
        let mut request = self.http.get(&url);
        if let Some(api_key) = &self.api_key {
            request = request.header("x-api-key", api_key);
        }
        
        let response = request.send().await?;
        let json: HashMap<String, PriceData> = response.json().await?;
        let price_data = json.get(token)
            .ok_or(anyhow::anyhow!("Token not found in response"))?;
        Ok(price_data.usd_price)
    }

    pub async fn get_quote(&self, input_mint: &str, output_mint: &str, amount: u64) -> 
    Result<QuoteResponse, anyhow::Error> {
        let url = format!(
            "https://public.jupiterapi.com/quote?inputMint={}&outputMint={}&amount={}&slippageBps=50",
            input_mint, output_mint, amount);
        
        let response = self.http.get(&url).send().await?;
        let quote: QuoteResponse = response.json().await?;

        Ok(quote)
    }

        pub async fn get_swap_tx(&self, quote: &QuoteResponse, user_pubkey: &str) -> Result<VersionedTransaction, anyhow::Error> {
        let url = format!("{}/swap", self.base_url);
        
        // Payload to send to Jupiter
        let body = serde_json::json!({
            "quoteResponse": quote,
            "userPublicKey": user_pubkey,
            "wrapAndUnwrapSol": true
        });

        // make request
        let resp = self.http.post(&url).json(&body).send().await?;
        let json: serde_json::Value = resp.json().await?;
        
        // Decode the base64 transaction string
        let swap_tx_base64 = json["swapTransaction"].as_str().unwrap();
        let tx_bytes = base64::engine::general_purpose::STANDARD.decode(swap_tx_base64)?;
        
        // Deserialize into a Solana Transaction object
        let tx: VersionedTransaction = bincode::deserialize(&tx_bytes)?;
        
        Ok(tx)
    }

        pub async fn execute_swap(&self, keypair: &Keypair, tx: VersionedTransaction) -> Result<String, anyhow::Error> {
        // Connect to Solana RPC
        let rpc_url = env::var("SOLANA_RPC_URL").unwrap();
        let rpc = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
        
        // Sign the transaction
        let mut signed_tx = tx;
        let latest_blockhash = rpc.get_latest_blockhash()?;
        
        // We act as the payer and signer
        signed_tx.message.set_recent_blockhash(latest_blockhash);
        let signature = keypair.sign_message(&signed_tx.message.serialize());
        signed_tx.signatures[0] = signature;

        // Send it!
        let signature = rpc.send_and_confirm_transaction(&signed_tx)?;
        Ok(signature.to_string())
    }


}



#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PriceData {
    pub usd_price: f64,
    pub block_id: u64,
    pub decimals: u8,
    pub price_change24h: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuoteResponse {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "swapMode")]
    pub swap_mode: String,
}

