use std::env;
use std::str::FromStr;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let rpc_url: String = env::var("RPC_URL").expect("RPC_URL environment variable is missing");
    let client: RpcClient = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    
    let private_key_str = env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable is missing");
    let payer: Keypair = Keypair::from_base58_string(&private_key_str); 
    println!("Using wallet: {}", payer.pubkey());

    // Protocol constants for Solend mainnet
    let program_id: Pubkey = Pubkey::from_str("So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo")?;
    let lending_market: Pubkey = Pubkey::from_str("4UpD2fh7xH3VP9QQaXtsS1YY3bxzWhtfpks7FatyKvdY")?;
    let market_authority: Pubkey = Pubkey::from_str("DdZR6zRFiUt4S5mg7AV1uKB2z1f1WzcNYCaTEEWPAuby")?;

    // USDC Reserve accounts
    let reserve_account: Pubkey = Pubkey::from_str("BgxfHJDzm44T7XG68MYKx7YisTjZu73tVovyZSjJMpmw")?; 
    let reserve_liquidity_supply: Pubkey = Pubkey::from_str("8SheGtsopRUDzdiD6v6BR9a6bqZ9QwywYQY99Fp5meNf")?; 
    let liquidity_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?; 
    
    let user_token_account = get_associated_token_address(&payer.pubkey(), &liquidity_mint);

    let fee_receiver: Pubkey = Pubkey::from_str("9RuqAN42PTUi9ya59k9suGATrkqzvb9gk2QABJtQzGP5")?;
    let fee_receiver_ata: Pubkey = get_associated_token_address(&fee_receiver, &liquidity_mint);

    let loan_amount: u64 = 10_000_000; // 10 USDC 
    
    // Ix 1 - FlashBorrow 
    let mut borrow_data = Vec::with_capacity(9);
    borrow_data.push(19); // discriminator 
    borrow_data.extend_from_slice(&loan_amount.to_le_bytes());

    let borrow_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(reserve_liquidity_supply, false),
            AccountMeta::new(user_token_account, false),
            AccountMeta::new(reserve_account, false),
            AccountMeta::new_readonly(lending_market, false),
            AccountMeta::new_readonly(market_authority, false),
            AccountMeta::new_readonly(solana_sdk::sysvar::instructions::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: borrow_data,
    };

    // Ix 2 - FlashRepay 
    let mut repay_data = Vec::with_capacity(10);
    repay_data.push(20); // discriminator
    repay_data.extend_from_slice(&loan_amount.to_le_bytes()); 
    repay_data.push(0); // Index of FlashBorrow ix in this tx

    let repay_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user_token_account, false),   // Source
            AccountMeta::new(reserve_liquidity_supply, false),   // Destination
            AccountMeta::new(fee_receiver_ata, false),  // Fee receiver
            AccountMeta::new(user_token_account, false),  // Host fee receiver (yourself)
            AccountMeta::new(reserve_account, false), // Reserve
            AccountMeta::new_readonly(lending_market, false),  // Lending Market
            AccountMeta::new_readonly(payer.pubkey(), true),   // Signer / Authority
            AccountMeta::new_readonly(solana_sdk::sysvar::instructions::id(), false),    // Instructions sysvar
            AccountMeta::new_readonly(spl_token::id(), false),     // Token Program ID
        ],
        data: repay_data,
    };

    let instructions = vec![borrow_ix, repay_ix];
    let recent_blockhash = client.get_latest_blockhash()?;

    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    println!("Simulating transaction...");
    let simulation = client.simulate_transaction(&transaction)?;

    match simulation.value.err {
        Some(err) => {
            println!("Simulation failed with error code: {:?}", err);
            if let Some(logs) = simulation.value.logs {
                println!("Program logs:");
                for log in logs {
                    println!("{}", log);
                }
            }
        }
        None => {
            println!("Simulation successful! Sending transaction to Mainnet...");
            match client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => println!("Success! Transaction hash: {}", signature),
                Err(err) => println!("Custom error: {:?}", err),
            }
        }
    }

    Ok(())
}

