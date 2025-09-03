//! Solana stub prover program that validates end_slot > start_slot
//! and commits to the public values matching the twine-solana-prover structure

#![no_main]
sp1_zkvm::entrypoint!(main);

use solana_stub_prover_lib::{ProverInput, PublicCommitments};
use sha2::{Sha256, Digest};

pub fn main() {
    // Read input from the prover
    let input = sp1_zkvm::io::read::<ProverInput>();
    
    // Simple validation: check that end_slot > start_slot
    assert!(input.end_slot > input.start_slot, "end_slot must be greater than start_slot");
    
    // Calculate a dummy account_data_hash from the monitored accounts
    let mut hasher = Sha256::new();
    for account in &input.monitored_accounts_state {
        hasher.update(&account.account_pubkey);
        hasher.update(&account.last_change_slot.to_le_bytes());
        hasher.update(&account.account_data_hash);
    }
    let account_data_hash: [u8; 32] = hasher.finalize().into();
    
    // Create dummy values for ESR and validator data
    let hash_root_valset = [0u8; 32]; // Dummy merkle root
    let total_active_stake = 1000000000u64; // 1 billion lamports
    let validator_count = 100u32; // 100 validators
    
    // Build public commitments
    let commitments = PublicCommitments {
        start_slot: input.start_slot,
        end_slot: input.end_slot,
        epoch: input.epoch,
        original_bank_hash: input.original_bank_hash,
        last_bank_hash: input.last_bank_hash,
        account_data_hash,
        hash_root_valset,
        total_active_stake,
        validator_count,
        monitored_accounts_state: input.monitored_accounts_state,
        validations_passed: true, // Always true for stub
    };
    
    // Serialize and commit the public values
    let bytes = bincode::serialize(&commitments).expect("Failed to serialize commitments");
    sp1_zkvm::io::commit_slice(&bytes);
}