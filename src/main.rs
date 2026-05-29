mod flashloan;
use std::str::FromStr;
use solana_sdk::pubkey::Pubkey;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let rpc_url: String = std::env::var("RPC_URL").expect("RPC_URL does not determine in .env");
    let private_key: String = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY does not determine in .env");
    let loan_amount: u64 = 1_000_000; 

    let reserve_account = Pubkey::from_str("BgxfHJDzm44T7XG68MYKx7YisTjZu73tVovyZSjJMpmw")?; 
    let reserve_liquidity_supply = Pubkey::from_str("8SheGtsopRUDzdiD6v6BR9a6bqZ9QwywYQY99Fp5meNf")?; 
    let liquidity_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?; 


    match flashloan::execute_flash_loan(&rpc_url, &private_key, loan_amount, reserve_account, reserve_liquidity_supply, liquidity_mint).await {
        Ok(_) => {},
        Err(e) => eprintln!("process crashed with error: {:?}", e),
    }
    Ok(())
}
