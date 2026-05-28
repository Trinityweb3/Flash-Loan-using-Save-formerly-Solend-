use std::str::FromStr;
use solana_client::nonblocking::rpc_client::RpcClient; // for async feature
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;

pub async fn execute_flash_loan(
    rpc_url: &str,
    private_key_str: &str,
    loan_amount: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let payer = Keypair::from_base58_string(private_key_str);
    println!("Using wallet: {}", payer.pubkey());

    let program_id = Pubkey::from_str("So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo")?;
    let lending_market = Pubkey::from_str("4UpD2fh7xH3VP9QQaXtsS1YY3bxzWhtfpks7FatyKvdY")?;
    let market_authority = Pubkey::from_str("DdZR6zRFiUt4S5mg7AV1uKB2z1f1WzcNYCaTEEWPAuby")?;

    let reserve_account = Pubkey::from_str("BgxfHJDzm44T7XG68MYKx7YisTjZu73tVovyZSjJMpmw")?; 
    let reserve_liquidity_supply = Pubkey::from_str("8SheGtsopRUDzdiD6v6BR9a6bqZ9QwywYQY99Fp5meNf")?; 
    let liquidity_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?; 
    
    let user_token_account = get_associated_token_address(&payer.pubkey(), &liquidity_mint);

    let fee_receiver = Pubkey::from_str("9RuqAN42PTUi9ya59k9suGATrkqzvb9gk2QABJtQzGP5")?;
    let fee_receiver_ata = get_associated_token_address(&fee_receiver, &liquidity_mint);

    let mut borrow_data = Vec::with_capacity(9);
    borrow_data.push(19); 
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

    let mut repay_data = Vec::with_capacity(10);
    repay_data.push(20); 
    repay_data.extend_from_slice(&loan_amount.to_le_bytes()); 
    repay_data.push(0); 

    let repay_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user_token_account, false),
            AccountMeta::new(reserve_liquidity_supply, false),
            AccountMeta::new(fee_receiver_ata, false),
            AccountMeta::new(user_token_account, false), 
            AccountMeta::new(reserve_account, false),
            AccountMeta::new_readonly(lending_market, false),
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::sysvar::instructions::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: repay_data,
    };

    let instructions = vec![borrow_ix, repay_ix];
    let recent_blockhash = client.get_latest_blockhash().await?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    println!("simulating transaction...");
    let simulation = client.simulate_transaction(&transaction).await?;

    match simulation.value.err {
        Some(err) => {
            println!("simulation failed with error code: {:?}", err);
            if let Some(logs) = simulation.value.logs {
                println!("Program logs:");
                for log in logs {
                    println!("  {}", log);
                }
            }
        }
        None => {
            println!("simulation successful! Sending to mainnet...");
            match client.send_and_confirm_transaction(&transaction).await {
                Ok(signature) => println!("success! Tx hash: {}", signature),
                Err(err) => println!("sending error: {:?}", err),
            }
        }
    }
    Ok(())
}
