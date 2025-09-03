use serde::Deserialize;

/// Solana RPC response for getAccountInfo
#[derive(Debug, Deserialize)]
pub struct AccountInfoResponse {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub result: AccountInfoResult,
}

#[derive(Debug, Deserialize)]
pub struct AccountInfoResult {
    pub context: Context,
    pub value: Option<AccountInfo>,
}

#[derive(Debug, Deserialize)]
pub struct Context {
    pub slot: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub data: Vec<String>,
    pub executable: bool,
    pub lamports: u64,
    pub owner: String,
    pub rent_epoch: u64,
    #[allow(dead_code)]
    pub space: u64,
}

/// Solana RPC response for getSlot
#[derive(Debug, Deserialize)]
pub struct SlotResponse {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub result: u64,
}