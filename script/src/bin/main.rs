//! Solana stub prover script that fetches account data from Solana devnet
//! and generates SP1 proofs to post to Kafka

use base64::{Engine as _, engine::general_purpose};
use clap::Parser;
use serde_json;
use std::fs;
use solana_stub_prover_lib::{ProverInput, PublicCommitments, AccountStateCommitment};
use solana_stub_prover_script::{
    kafka::{publish_json_to_kafka_with_config, KafkaConfig},
    solana::{fetch_account_info, get_current_slot},
    utils::{base58_to_bytes32, get_epoch_for_slot, sha256_from_u64, sha256_hash},
};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};

/// The ELF file for the Solana stub prover program
pub const PROVER_ELF: &[u8] = include_elf!("solana-stub-prover-program");

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Start slot number
    #[arg(long)]
    start_slot: u64,
    
    /// End slot number
    #[arg(long)]
    end_slot: u64,
    
    /// Account pubkey to monitor (base58 encoded)
    #[arg(long)]
    account: String,
    
    /// Execute only (no proof generation)
    #[arg(long)]
    execute: bool,
    
    /// Generate proof
    #[arg(long)]
    prove: bool,
    
    /// Use current slot if not specified (optional)
    #[arg(long)]
    use_current_slot: bool,
    
    /// Generate Groth16 proof for on-chain verification (default: true)
    #[arg(long, default_value = "true")]
    groth16: bool,
    
    /// Generate compressed proof only (faster, but not verifiable on-chain)
    #[arg(long)]
    compressed_only: bool,
    
    /// Use TLS for Kafka connection (default: true)
    #[arg(long, default_value = "true")]
    kafka_tls: bool,
    
    /// CA certificate file path for Kafka TLS
    #[arg(long, default_value = "./ca.crt")]
    kafka_ca_cert: String,
    
    /// Client certificate file path for Kafka TLS
    #[arg(long, default_value = "./user.crt")]
    kafka_client_cert: String,
    
    /// Client key file path for Kafka TLS
    #[arg(long, default_value = "./user.key")]
    kafka_client_key: String,
    
    /// Kafka broker address (overrides default)
    #[arg(long)]
    kafka_broker: Option<String>,
    
    /// Disable Kafka TLS (use plain connection)
    #[arg(long)]
    no_kafka_tls: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logger
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();
    
    // Parse arguments
    let mut args = Args::parse();
    
    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }
    
    // Optionally use current slot
    if args.use_current_slot && args.end_slot == 0 {
        args.end_slot = get_current_slot().await?;
        println!("Using current slot as end_slot: {}", args.end_slot);
    }
    
    println!("Fetching account info for: {}", args.account);
    println!("Start slot: {}, End slot: {}", args.start_slot, args.end_slot);
    
    // Validate slots
    if args.end_slot <= args.start_slot {
        eprintln!("Error: end_slot must be greater than start_slot");
        std::process::exit(1);
    }
    
    // Fetch account info
    let (account_info, actual_slot) = fetch_account_info(&args.account, Some(args.end_slot)).await?;
    println!("Fetched account info at slot: {}", actual_slot);
    
    // Use the actual slot if it's different from requested
    let effective_end_slot = if actual_slot > args.end_slot {
        println!("Note: Using actual slot {} as end_slot (was {})", actual_slot, args.end_slot);
        actual_slot
    } else {
        args.end_slot
    };
    
    // Decode account data
    let account_data = if !account_info.data.is_empty() {
        general_purpose::STANDARD.decode(&account_info.data[0])?
    } else {
        Vec::new()
    };
    
    // Convert account pubkey
    let account_pubkey = base58_to_bytes32(&args.account)?;
    
    // Convert owner pubkey
    let owner_bytes = base58_to_bytes32(&account_info.owner)?;
    
    // Calculate account data hash
    let account_data_hash = sha256_hash(&account_data);
    
    // Get epoch for the actual slot
    let epoch = get_epoch_for_slot(effective_end_slot);
    
    // Create account state commitment with actual slot data
    let account_state = AccountStateCommitment {
        account_pubkey,
        last_change_slot: effective_end_slot,
        account_data_hash,
        lamports: account_info.lamports,
        owner: owner_bytes,
        executable: account_info.executable,
        rent_epoch: account_info.rent_epoch,
        data: account_data,
    };
    
    // Create dummy bank hashes
    let original_bank_hash = sha256_from_u64(args.start_slot);
    let last_bank_hash = sha256_from_u64(effective_end_slot);
    
    // Create prover input with effective end slot
    let input = ProverInput {
        start_slot: args.start_slot,
        end_slot: effective_end_slot,
        epoch,
        original_bank_hash,
        last_bank_hash,
        monitored_accounts_state: vec![account_state],
    };
    
    // Setup prover client
    let client = ProverClient::from_env();
    
    // Prepare input
    let mut stdin = SP1Stdin::new();
    stdin.write(&input);
    
    if args.execute {
        // Execute only
        let (output, report) = client.execute(PROVER_ELF, &stdin).run().unwrap();
        println!("Program executed successfully.");
        
        // Deserialize output
        let commitments: PublicCommitments = bincode::deserialize(&output.to_vec()).unwrap();
        println!("Commitments: {:?}", commitments);
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        // Generate proof
        println!("Setting up proving keys...");
        let (pk, vk) = client.setup(PROVER_ELF);
        
        // Save verification key to file
        let vkey_json = serde_json::to_string_pretty(&vk).expect("Failed to serialize verification key");
        fs::write("vkey.json", &vkey_json).expect("Failed to write vkey.json");
        println!("Verification key saved to vkey.json ({} bytes)", vkey_json.len());
        
        if args.compressed_only {
            // Generate compressed proof only (faster but not verifiable on-chain)
            println!("Generating compressed proof...");
            let proof = client
                .prove(&pk, &stdin)
                .compressed()
                .run()
                .expect("failed to generate compressed proof");
            
            println!("Successfully generated compressed proof!");
            
            // Verify the compressed proof
            client.verify(&proof, &vk).expect("failed to verify proof");
            println!("Successfully verified compressed proof!");
            
            // Serialize the proof to JSON
            let proof_json = serde_json::to_string_pretty(&proof).expect("Failed to serialize proof");
            println!("Proof size (JSON): {} bytes", proof_json.len());
            
            // Save proof to file
            fs::write("last_proof.json", &proof_json).expect("Failed to write last_proof.json");
            println!("Proof saved to last_proof.json");
            
            // Create ZkProof structure for Kafka
            let zk_proof = serde_json::json!({
                "identifier": format!("solana-stub-{}-{}", args.start_slot, effective_end_slot),
                "kind": "SolanaConsensusProof",
                "proof_data": {
                    "type": "SP1_Compressed",
                    "version": 1,
                    "proof": serde_json::from_str::<serde_json::Value>(&proof_json).unwrap(),
                    "public_values": hex::encode(proof.public_values.to_vec()),
                }
            });
            
            // Save full ZkProof structure to file as well
            let zk_proof_json = serde_json::to_string_pretty(&zk_proof).expect("Failed to serialize ZkProof");
            fs::write("last_kafka_message.json", &zk_proof_json).expect("Failed to write last_kafka_message.json");
            println!("Full Kafka message saved to last_kafka_message.json");
            
            // Configure Kafka
            let kafka_config = KafkaConfig {
                use_tls: !args.no_kafka_tls && args.kafka_tls,
                ca_cert_path: Some(args.kafka_ca_cert.clone()),
                client_cert_path: Some(args.kafka_client_cert.clone()),
                client_key_path: Some(args.kafka_client_key.clone()),
                broker: args.kafka_broker.clone(),
            };
            
            // Publish to Kafka as JSON
            println!("Publishing compressed proof to Kafka...");
            publish_json_to_kafka_with_config(zk_proof, &kafka_config).await?;
            println!("Compressed proof successfully published to Kafka!");
        } else {
            // Generate Groth16 proof for on-chain verification (default)
            println!("Generating Groth16 proof...");
            let groth16_proof = client
                .prove(&pk, &stdin)
                .groth16()
                .run()
                .expect("failed to generate Groth16 proof");
            
            println!("Successfully generated Groth16 proof!");
            
            // Serialize the Groth16 proof to JSON using native serde
            let proof_json = serde_json::to_string_pretty(&groth16_proof).expect("Failed to serialize Groth16 proof");
            println!("Groth16 proof size (JSON): {} bytes", proof_json.len());
            
            // Save proof to file
            fs::write("last_proof.json", &proof_json).expect("Failed to write last_proof.json");
            println!("Groth16 proof saved to last_proof.json");
            
            // Extract public values if available
            let public_values_hex = if let Ok(public_values) = serde_json::to_value(&groth16_proof) {
                if let Some(pv) = public_values.get("public_values") {
                    pv.to_string()
                } else {
                    // If no public_values field, encode the whole proof
                    hex::encode(bincode::serialize(&groth16_proof).unwrap_or_default())
                }
            } else {
                String::from("0x")
            };
            
            // Create ZkProof structure as JSON
            let zk_proof = serde_json::json!({
                "identifier": format!("solana-stub-{}-{}", args.start_slot, effective_end_slot),
                "kind": "SolanaConsensusProof",
                "proof_data": {
                    "type": "SP1_Groth16",
                    "version": 2,
                    "proof": serde_json::from_str::<serde_json::Value>(&proof_json).unwrap(),
                    "public_values": public_values_hex,
                    "groth16": true
                }
            });
            
            // Save full ZkProof structure to file as well
            let zk_proof_json = serde_json::to_string_pretty(&zk_proof).expect("Failed to serialize ZkProof");
            fs::write("last_kafka_message.json", &zk_proof_json).expect("Failed to write last_kafka_message.json");
            println!("Full Kafka message saved to last_kafka_message.json");
            
            // Configure Kafka
            let kafka_config = KafkaConfig {
                use_tls: !args.no_kafka_tls && args.kafka_tls,
                ca_cert_path: Some(args.kafka_ca_cert.clone()),
                client_cert_path: Some(args.kafka_client_cert.clone()),
                client_key_path: Some(args.kafka_client_key.clone()),
                broker: args.kafka_broker.clone(),
            };
            
            // Publish to Kafka as JSON
            println!("Publishing Groth16 proof to Kafka...");
            publish_json_to_kafka_with_config(zk_proof, &kafka_config).await?;
            println!("Groth16 proof successfully published to Kafka!");
        }
    }
    
    Ok(())
}