//! Kafka consumer that listens to the twine.solana.proofs topic and prints messages

use clap::Parser;
use rdkafka::consumer::{StreamConsumer, Consumer};
use rdkafka::{ClientConfig, Message};
use rdkafka::message::Headers;
use serde_json::Value;
use solana_stub_prover_script::types::{ZkProof, ProofData};
use futures::StreamExt;
use chrono::Utc;
use std::time::Duration;

const DEFAULT_KAFKA_BROKER: &str = "b-1.test.7alql0.c5.kafka.us-east-1.amazonaws.com:9092";
const KAFKA_TOPIC: &str = "twine.solana.proofs";

/// Command line arguments for the consumer
#[derive(Parser, Debug)]
#[command(author, version, about = "Kafka consumer for Solana proofs", long_about = None)]
struct Args {
    /// Kafka broker address (can be comma-separated list)
    #[arg(long, default_value = DEFAULT_KAFKA_BROKER)]
    broker: String,
    
    /// Consumer group ID
    #[arg(long, default_value = "solana-proof-consumer")]
    group_id: String,
    
    /// Start from beginning of topic
    #[arg(long)]
    from_beginning: bool,
    
    /// Show raw JSON output
    #[arg(long)]
    raw: bool,
    
    /// Show only proof identifiers (minimal output)
    #[arg(long)]
    minimal: bool,
    
    /// Enable SASL authentication
    #[arg(long)]
    sasl: bool,
    
    /// SASL username
    #[arg(long, env = "KAFKA_USERNAME")]
    username: Option<String>,
    
    /// SASL password
    #[arg(long, env = "KAFKA_PASSWORD")]
    password: Option<String>,
    
    /// SASL mechanism (PLAIN, SCRAM-SHA-256, SCRAM-SHA-512)
    #[arg(long, default_value = "PLAIN")]
    sasl_mechanism: String,
    
    /// Security protocol (plaintext, ssl, sasl_plaintext, sasl_ssl)
    #[arg(long, default_value = "plaintext")]
    security_protocol: String,
    
    /// Enable debug output
    #[arg(long)]
    debug: bool,
    
    /// Connection timeout in seconds
    #[arg(long, default_value = "30")]
    connection_timeout: u64,
}

fn format_bytes(bytes: &[u8], max_len: usize) -> String {
    if bytes.len() <= max_len {
        hex::encode(bytes)
    } else {
        format!("{}... ({} bytes)", hex::encode(&bytes[..max_len]), bytes.len())
    }
}

fn print_proof_details(proof: &ZkProof, raw: bool, minimal: bool) {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    
    if minimal {
        println!("[{}] Proof ID: {}", timestamp, proof.identifier);
        return;
    }
    
    if raw {
        // Print raw JSON
        match serde_json::to_string_pretty(proof) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Error serializing proof: {}", e),
        }
        println!("---");
        return;
    }
    
    // Pretty print proof details
    println!("\n╔══════════════════════════════════════════════════════════════════════");
    println!("║ 📦 New Proof Received at {}", timestamp);
    println!("╟──────────────────────────────────────────────────────────────────────");
    println!("║ Identifier: {}", proof.identifier);
    println!("║ Proof Kind: {:?}", proof.proof_kind);
    
    match &proof.proof_data {
        ProofData::SP1(sp1_proof) => {
            println!("║ Proof Type: SP1");
            println!("║ Version: {}", sp1_proof.version);
            println!("║ Verification Key: {}", hex::encode(&sp1_proof.verification_key));
            println!("║ Proof Size: {} bytes", sp1_proof.proof.len());
            println!("║ Public Values Size: {} bytes", sp1_proof.public_value.len());
            
            // Try to decode public values as PublicCommitments
            if let Ok(commitments) = bincode::deserialize::<solana_stub_prover_lib::PublicCommitments>(&sp1_proof.public_value) {
                println!("║");
                println!("║ 📊 Public Commitments:");
                println!("║   Start Slot: {}", commitments.start_slot);
                println!("║   End Slot: {}", commitments.end_slot);
                println!("║   Epoch: {}", commitments.epoch);
                println!("║   Original Bank Hash: {}", format_bytes(&commitments.original_bank_hash, 8));
                println!("║   Last Bank Hash: {}", format_bytes(&commitments.last_bank_hash, 8));
                println!("║   Account Data Hash: {}", format_bytes(&commitments.account_data_hash, 8));
                println!("║   Validator Set Hash: {}", format_bytes(&commitments.hash_root_valset, 8));
                println!("║   Total Active Stake: {}", commitments.total_active_stake);
                println!("║   Validator Count: {}", commitments.validator_count);
                println!("║   Monitored Accounts: {}", commitments.monitored_accounts_state.len());
                println!("║   Validations Passed: {}", commitments.validations_passed);
                
                for (i, account) in commitments.monitored_accounts_state.iter().enumerate() {
                    println!("║");
                    println!("║   Account #{}: {}", i + 1, format_bytes(&account.account_pubkey, 8));
                    println!("║     Last Change Slot: {}", account.last_change_slot);
                    println!("║     Lamports: {}", account.lamports);
                    println!("║     Executable: {}", account.executable);
                    println!("║     Data Size: {} bytes", account.data.len());
                }
            } else {
                println!("║   (Unable to decode public commitments)");
            }
        }
    }
    
    println!("╚══════════════════════════════════════════════════════════════════════");
}

async fn test_connection(broker: &str, timeout_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Testing connection to broker: {}", broker);
    
    let mut test_config = ClientConfig::new();
    let test_consumer: Result<StreamConsumer, _> = test_config
        .set("bootstrap.servers", broker)
        .set("group.id", "connection-test")
        .set("socket.timeout.ms", &format!("{}", timeout_secs * 1000))
        .set("session.timeout.ms", "6000")
        .create();
    
    match test_consumer {
        Ok(consumer) => {
            // Try to get metadata (this is a synchronous call)
            match consumer.fetch_metadata(None, Duration::from_secs(timeout_secs)) {
                Ok(metadata) => {
                    println!("✅ Successfully connected to broker");
                    println!("   Broker count: {}", metadata.brokers().len());
                    for broker in metadata.brokers() {
                        println!("   - Broker {}: {}:{}", broker.id(), broker.host(), broker.port());
                    }
                    Ok(())
                }
                Err(e) => {
                    Err(format!("Failed to fetch metadata: {}", e).into())
                }
            }
        }
        Err(e) => {
            Err(format!("Failed to create test consumer: {}", e).into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments
    let args = Args::parse();
    
    println!("🚀 Starting Kafka Consumer");
    println!("📍 Broker(s): {}", args.broker);
    println!("📨 Topic: {}", KAFKA_TOPIC);
    println!("👥 Group ID: {}", args.group_id);
    println!("🔐 Security Protocol: {}", args.security_protocol);
    
    if args.sasl {
        println!("🔑 SASL Authentication: Enabled");
        println!("   Mechanism: {}", args.sasl_mechanism);
        if args.username.is_some() {
            println!("   Username: ***");
        }
    }
    
    if args.from_beginning {
        println!("⏮️  Reading from beginning of topic");
    }
    if args.minimal {
        println!("📝 Minimal output mode");
    } else if args.raw {
        println!("📝 Raw JSON output mode");
    }
    
    println!("────────────────────────────────────────────────────────────────────");
    
    // Test connection first
    if let Err(e) = test_connection(&args.broker, args.connection_timeout).await {
        eprintln!("\n❌ Connection test failed: {}", e);
        eprintln!("\n🔍 Troubleshooting tips:");
        eprintln!("   1. Check if the broker address is correct: {}", args.broker);
        eprintln!("   2. Verify network connectivity to the broker");
        eprintln!("   3. Check if authentication is required (use --sasl flag)");
        eprintln!("   4. Ensure the broker port is not blocked by firewall");
        eprintln!("   5. Try using a different security protocol (--security-protocol)");
        eprintln!("\nYou can also try:");
        eprintln!("   - Using localhost:9092 for local Kafka");
        eprintln!("   - Setting KAFKA_USERNAME and KAFKA_PASSWORD environment variables");
        eprintln!("   - Using --debug flag for more detailed error messages");
        
        if !args.debug {
            return Err(e);
        }
        
        eprintln!("\n⚠️  Debug mode enabled, continuing despite connection test failure...");
    }
    
    // Create consumer configuration
    let mut config = ClientConfig::new();
    config
        .set("bootstrap.servers", &args.broker)
        .set("group.id", &args.group_id)
        .set("enable.auto.commit", "true")
        .set("auto.commit.interval.ms", "1000")
        .set("session.timeout.ms", "6000")
        .set("socket.timeout.ms", &format!("{}", args.connection_timeout * 1000))
        .set("api.version.request.timeout.ms", "10000")
        .set("enable.auto.offset.store", "true")
        .set("security.protocol", &args.security_protocol);
    
    // Add SASL configuration if enabled
    if args.sasl {
        config.set("sasl.mechanism", &args.sasl_mechanism);
        
        if let Some(username) = &args.username {
            config.set("sasl.username", username);
        }
        
        if let Some(password) = &args.password {
            config.set("sasl.password", password);
        }
    }
    
    // Debug settings
    if args.debug {
        config.set("debug", "all");
    }
    
    if args.from_beginning {
        config.set("auto.offset.reset", "earliest");
    } else {
        config.set("auto.offset.reset", "latest");
    }
    
    // Create consumer
    let consumer: StreamConsumer = match config.create() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to create consumer: {}", e);
            eprintln!("\n💡 Possible solutions:");
            eprintln!("   - Check broker connectivity: telnet {} <port>", args.broker.split(':').next().unwrap_or(&args.broker));
            eprintln!("   - Verify Kafka is running on the specified broker");
            eprintln!("   - Check authentication credentials if SASL is enabled");
            return Err(Box::new(e));
        }
    };
    
    // Subscribe to topic
    match consumer.subscribe(&[KAFKA_TOPIC]) {
        Ok(_) => println!("✅ Subscribed to topic: {}", KAFKA_TOPIC),
        Err(e) => {
            eprintln!("❌ Failed to subscribe to topic: {}", e);
            return Err(Box::new(e));
        }
    }
    
    println!("⏳ Waiting for messages... (Press Ctrl+C to stop)\n");
    
    // Process messages
    let mut message_stream = consumer.stream();
    let mut message_count = 0;
    let mut error_count = 0;
    const MAX_CONSECUTIVE_ERRORS: u32 = 10;
    
    while let Some(message) = message_stream.next().await {
        match message {
            Ok(msg) => {
                error_count = 0; // Reset error counter on success
                message_count += 1;
                
                // Get message details
                let key = msg.key().map(|k| String::from_utf8_lossy(k).to_string())
                    .unwrap_or_else(|| "no-key".to_string());
                
                let partition = msg.partition();
                let offset = msg.offset();
                
                if !args.minimal && !args.raw {
                    println!("📬 Message #{} | Partition: {} | Offset: {} | Key: {}", 
                        message_count, partition, offset, key);
                }
                
                // Parse message payload
                if let Some(payload) = msg.payload() {
                    match serde_json::from_slice::<ZkProof>(payload) {
                        Ok(proof) => {
                            print_proof_details(&proof, args.raw, args.minimal);
                        }
                        Err(e) => {
                            // Try to parse as generic JSON for debugging
                            if let Ok(json) = serde_json::from_slice::<Value>(payload) {
                                eprintln!("⚠️  Could not parse as ZkProof, showing raw JSON:");
                                println!("{}", serde_json::to_string_pretty(&json)?);
                            } else {
                                eprintln!("❌ Error parsing message: {}", e);
                                eprintln!("   Raw payload: {}", String::from_utf8_lossy(payload));
                            }
                        }
                    }
                } else {
                    eprintln!("⚠️  Empty message payload");
                }
                
                // Print headers if present and not in minimal mode
                if !args.minimal && !args.raw {
                    if let Some(headers) = msg.headers() {
                        for header in headers.iter() {
                            println!("   Header: {} = {}", 
                                header.key, 
                                String::from_utf8_lossy(header.value.unwrap_or(b"")));
                        }
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                eprintln!("❌ Error receiving message (attempt {}/{}): {}", 
                    error_count, MAX_CONSECUTIVE_ERRORS, e);
                
                if error_count >= MAX_CONSECUTIVE_ERRORS {
                    eprintln!("\n⚠️  Too many consecutive errors. The broker connection may be lost.");
                    eprintln!("   Last error: {}", e);
                    eprintln!("\n🔍 Troubleshooting:");
                    eprintln!("   - Check network connectivity");
                    eprintln!("   - Verify the Kafka broker is still running");
                    eprintln!("   - Check if authentication has expired");
                    eprintln!("   - Review broker logs for issues");
                    
                    return Err(format!("Consumer stopped after {} consecutive errors", MAX_CONSECUTIVE_ERRORS).into());
                }
                
                // Wait a bit before retrying
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
    
    println!("\n👋 Consumer shutting down. Processed {} messages.", message_count);
    Ok(())
}