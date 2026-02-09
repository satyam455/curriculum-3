mod jupiter; // Tell Rust we have a jupiter.rs file

#[tokio::main]
async fn main() {

    let client = jupiter::JupiterClient::new();
    println!("Starting Jupiter Client...");
    
    println!("Client created! Base URL: {}", client.base_url);

    // SOL token address on Solana
    match client.get_price("So11111111111111111111111111111111111111112").await {
        Ok(price) => println!("Current SOL price: ${}", price),
        Err(e) => println!("Error: {}", e),
    }

    // SOL Mint Address
    let sol_mint = "So11111111111111111111111111111111111111112";
    // USDC Mint Address
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    
    // Get quote for 0.1 SOL (100,000,000 lamports)
    let quote = client.get_quote(sol_mint, usdc_mint, 100_000_000).await.unwrap();
    
    println!("Swap Quote:");
    println!("Input: {} lamports", quote.in_amount);
    println!("Output: {} micro-USDC", quote.out_amount);


}
