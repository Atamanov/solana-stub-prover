use crate::types::{AccountInfo, AccountInfoResponse, SlotResponse};
use reqwest;
use serde_json::json;
use std::error::Error;

const DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";

/// Fetch account information from Solana RPC
/// Note: Solana RPC may return data from a more recent slot than requested
pub async fn fetch_account_info(
    account: &str,
    slot: Option<u64>,
) -> Result<(AccountInfo, u64), Box<dyn Error>> {
    let client = reqwest::Client::new();
    
    // Build params based on whether we want a specific slot
    let params = if let Some(target_slot) = slot {
        // Request account info with minContextSlot to ensure we get data at or after the target slot
        json!([
            account,
            {
                "encoding": "base64",
                "commitment": "confirmed",
                "minContextSlot": target_slot
            }
        ])
    } else {
        json!([
            account,
            {
                "encoding": "base64",
                "commitment": "confirmed"
            }
        ])
    };
    
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": params
    });
    
    let response = client
        .post(DEVNET_RPC_URL)
        .json(&request)
        .send()
        .await?;
    
    let account_response: AccountInfoResponse = response.json().await?;
    
    let actual_slot = account_response.result.context.slot;
    
    // Warn if we got data from a different slot than requested
    if let Some(target_slot) = slot {
        if actual_slot != target_slot {
            eprintln!(
                "Warning: Requested slot {} but got data from slot {} (difference: {})",
                target_slot,
                actual_slot,
                actual_slot as i64 - target_slot as i64
            );
            eprintln!("Note: Solana RPC returns the latest available data, historical slot data may not be available");
        }
    }
    
    match account_response.result.value {
        Some(account_info) => Ok((account_info, actual_slot)),
        None => Err("Account not found".into()),
    }
}

/// Get current slot from Solana RPC
pub async fn get_current_slot() -> Result<u64, Box<dyn Error>> {
    let client = reqwest::Client::new();
    
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getSlot",
        "params": [{"commitment": "confirmed"}]
    });
    
    let response = client
        .post(DEVNET_RPC_URL)
        .json(&request)
        .send()
        .await?;
    
    let slot_response: SlotResponse = response.json().await?;
    Ok(slot_response.result)
}