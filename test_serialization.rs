use twine_types::proofs::{ZkProof, ProofKind, ProofData, SP1Proof};
use serde_json;

fn main() {
    // Create a small test proof
    let sp1_proof = SP1Proof {
        version: 1,
        proof: vec![1, 2, 3, 4, 5],  // Small proof for testing
        public_value: vec![10, 20, 30],
        verification_key: [0u8; 32],
    };
    
    let zk_proof = ZkProof {
        identifier: "test-proof-123".to_string(),
        proof_kind: ProofKind::SolanaConsensusProof,
        proof_data: ProofData::SP1(sp1_proof),
    };
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(&zk_proof).unwrap();
    println!("JSON structure:");
    println!("{}", json);
    println!("\nJSON size: {} bytes", json.len());
}