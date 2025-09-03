use bs58;
use sha2::{Sha256, Digest};

/// Decode a base58 string to bytes
pub fn base58_decode(input: &str) -> Result<Vec<u8>, bs58::decode::Error> {
    bs58::decode(input).into_vec()
}

/// Convert a base58 public key to 32-byte array
pub fn base58_to_bytes32(pubkey: &str) -> Result<[u8; 32], String> {
    let bytes = base58_decode(pubkey).map_err(|e| e.to_string())?;
    if bytes.len() != 32 {
        return Err(format!("Invalid pubkey length: {}", bytes.len()));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

/// Calculate SHA256 hash and return as 32-byte array
pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Calculate SHA256 hash from a u64 value
pub fn sha256_from_u64(value: u64) -> [u8; 32] {
    sha256_hash(&value.to_le_bytes())
}

/// Calculate epoch number from slot
pub fn get_epoch_for_slot(slot: u64) -> u64 {
    // Solana mainnet/devnet has 432000 slots per epoch
    const SLOTS_PER_EPOCH: u64 = 432000;
    slot / SLOTS_PER_EPOCH
}