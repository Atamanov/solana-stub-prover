use serde::{Deserialize, Serialize};

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

/// SP1 Proof structure matching weaver types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SP1Proof {
    pub version: u64,
    pub proof: Vec<u8>,
    pub public_value: Vec<u8>,
    pub verification_key: [u8; 32],
}

/// ZkProof structure matching weaver types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProof {
    pub identifier: String,
    #[serde(rename = "kind")]
    pub proof_kind: ProofKind,
    pub proof_data: ProofData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofKind {
    ExecutionProof(u64),
    SolanaConsensusProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofData {
    SP1(SP1Proof),
}