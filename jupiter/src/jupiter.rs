use reqwest::Client;

use serde::Deserialize;
use std::collections::HashMap;

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
}



#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PriceData {
    pub usd_price: f64,
    pub block_id: u64,
    pub decimals: u8,
    pub price_change24h: f64,
}

#[derive(Deserialize, Debug)]
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

