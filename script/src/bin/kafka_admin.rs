//! Kafka admin tool to check and create topics

use clap::{Parser, Subcommand};
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::metadata::Metadata;
use std::time::Duration;

const DEFAULT_KAFKA_BROKER_TLS: &str = "kafka-bootstrap.twine.limited:443";
const DEFAULT_KAFKA_BROKER_PLAIN: &str = "b-1.test.7alql0.c5.kafka.us-east-1.amazonaws.com:9092";
const KAFKA_TOPIC: &str = "twine.solana.proofs";

#[derive(Parser, Debug)]
#[command(author, version, about = "Kafka admin tool for managing topics", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
    
    /// Kafka broker address
    #[arg(long)]
    broker: Option<String>,
    
    /// Use TLS connection (default: true)
    #[arg(long, default_value = "true")]
    tls: bool,
    
    /// Disable TLS (use plain connection)
    #[arg(long)]
    no_tls: bool,
    
    /// CA certificate file path
    #[arg(long, default_value = "./ca.crt")]
    ca_cert: String,
    
    /// Client certificate file path  
    #[arg(long, default_value = "./user.crt")]
    client_cert: String,
    
    /// Client key file path
    #[arg(long, default_value = "./user.key")]
    client_key: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all topics
    List,
    
    /// Check if a specific topic exists
    Check {
        /// Topic name to check
        #[arg(long, default_value = KAFKA_TOPIC)]
        topic: String,
    },
    
    /// Create a new topic
    Create {
        /// Topic name to create
        #[arg(long, default_value = KAFKA_TOPIC)]
        topic: String,
        
        /// Number of partitions
        #[arg(long, default_value = "3")]
        partitions: i32,
        
        /// Replication factor
        #[arg(long, default_value = "1")]
        replication_factor: i32,
    },
    
    /// Get metadata about topics
    Metadata {
        /// Specific topic to describe (optional)
        #[arg(long)]
        topic: Option<String>,
    },
}

fn create_admin_client(args: &Args) -> Result<AdminClient<DefaultClientContext>, rdkafka::error::KafkaError> {
    let mut config = ClientConfig::new();
    
    // Determine if TLS should be used
    let use_tls = !args.no_tls && args.tls;
    
    // Determine broker address
    let broker = args.broker.as_ref().map(|s| s.as_str()).unwrap_or_else(|| {
        if use_tls {
            DEFAULT_KAFKA_BROKER_TLS
        } else {
            DEFAULT_KAFKA_BROKER_PLAIN
        }
    });
    
    config.set("bootstrap.servers", broker);
    
    // Configure TLS if enabled
    if use_tls {
        config.set("security.protocol", "ssl");
        config.set("ssl.ca.location", &args.ca_cert);
        config.set("ssl.certificate.location", &args.client_cert);
        config.set("ssl.key.location", &args.client_key);
        println!("🔐 Using TLS connection to {}", broker);
    } else {
        println!("📡 Using plain connection to {}", broker);
    }
    
    config.create()
}

fn print_metadata(metadata: &Metadata, topic_filter: Option<&str>) {
    println!("\n📊 Cluster Metadata:");
    println!("   Broker count: {}", metadata.brokers().len());
    
    for broker in metadata.brokers() {
        println!("   - Broker {}: {}:{}", broker.id(), broker.host(), broker.port());
    }
    
    println!("\n📋 Topics:");
    for topic in metadata.topics() {
        if let Some(filter) = topic_filter {
            if topic.name() != filter {
                continue;
            }
        }
        
        println!("\n   Topic: {}", topic.name());
        println!("   Partitions: {}", topic.partitions().len());
        
        for partition in topic.partitions() {
            println!("      - Partition {}: Leader: {}, Replicas: {:?}, ISR: {:?}",
                partition.id(),
                partition.leader(),
                partition.replicas(),
                partition.isr()
            );
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    println!("🚀 Kafka Admin Tool");
    println!("────────────────────────────────────────────");
    
    let admin = match create_admin_client(&args) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("❌ Failed to create admin client: {}", e);
            std::process::exit(1);
        }
    };
    
    match args.command {
        Commands::List => {
            println!("\n📋 Fetching topic list...");
            
            match admin.inner().fetch_metadata(None, Duration::from_secs(10)) {
                Ok(metadata) => {
                    println!("\n✅ Topics in cluster:");
                    for topic in metadata.topics() {
                        println!("   - {}", topic.name());
                    }
                    
                    if metadata.topics().is_empty() {
                        println!("   (No topics found)");
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to fetch metadata: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Check { topic } => {
            println!("\n🔍 Checking if topic '{}' exists...", topic);
            
            match admin.inner().fetch_metadata(None, Duration::from_secs(10)) {
                Ok(metadata) => {
                    let exists = metadata.topics().iter().any(|t| t.name() == topic);
                    
                    if exists {
                        println!("✅ Topic '{}' exists", topic);
                        
                        // Get detailed info about the topic
                        match admin.inner().fetch_metadata(Some(&topic), Duration::from_secs(10)) {
                            Ok(topic_metadata) => {
                                print_metadata(&topic_metadata, Some(&topic));
                            }
                            Err(e) => {
                                eprintln!("⚠️  Failed to fetch topic metadata: {}", e);
                            }
                        }
                    } else {
                        println!("❌ Topic '{}' does not exist", topic);
                        println!("\n💡 To create it, run:");
                        println!("   cargo run --release --bin kafka_admin -- create --topic {}", topic);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to fetch metadata: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Create { topic, partitions, replication_factor } => {
            println!("\n📝 Creating topic '{}'...", topic);
            println!("   Partitions: {}", partitions);
            println!("   Replication factor: {}", replication_factor);
            
            let new_topic = NewTopic::new(
                &topic,
                partitions,
                TopicReplication::Fixed(replication_factor)
            );
            
            let options = AdminOptions::new().operation_timeout(Some(Duration::from_secs(30)));
            
            match admin.create_topics(&[new_topic], &options).await {
                Ok(results) => {
                    for result in results {
                        match result {
                            Ok(name) => println!("✅ Topic '{}' created successfully", name),
                            Err((name, err)) => {
                                if err.to_string().contains("already exists") {
                                    println!("⚠️  Topic '{}' already exists", name);
                                } else {
                                    eprintln!("❌ Failed to create topic '{}': {}", name, err);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to create topics: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Metadata { topic } => {
            println!("\n📊 Fetching cluster metadata...");
            
            match admin.inner().fetch_metadata(topic.as_deref(), Duration::from_secs(10)) {
                Ok(metadata) => {
                    print_metadata(&metadata, topic.as_deref());
                }
                Err(e) => {
                    eprintln!("❌ Failed to fetch metadata: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}