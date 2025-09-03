use twine_types::proofs::ZkProof;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde_json::Value;
use std::error::Error;
use std::time::Duration;
use std::path::Path;

const KAFKA_BROKER_TLS: &str = "kafka-bootstrap.twine.limited:443";
const KAFKA_BROKER_PLAIN: &str = "b-1.test.7alql0.c5.kafka.us-east-1.amazonaws.com:9092";
const KAFKA_TOPIC: &str = "twine.solana.proofs";

/// Kafka configuration options
pub struct KafkaConfig {
    pub use_tls: bool,
    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub broker: Option<String>,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            use_tls: true, // Default to TLS
            ca_cert_path: Some("./ca.crt".to_string()),
            client_cert_path: Some("./user.crt".to_string()),
            client_key_path: Some("./user.key".to_string()),
            broker: None,
        }
    }
}

/// Create a Kafka producer with the given configuration
pub fn create_producer(config: &KafkaConfig) -> Result<FutureProducer, Box<dyn Error>> {
    let mut client_config = ClientConfig::new();
    
    // Determine broker address
    let broker = config.broker.as_ref().map(|s| s.as_str()).unwrap_or_else(|| {
        if config.use_tls {
            KAFKA_BROKER_TLS
        } else {
            KAFKA_BROKER_PLAIN
        }
    });
    
    client_config.set("bootstrap.servers", broker);
    client_config.set("message.timeout.ms", "5000");
    
    // Configure TLS if enabled
    if config.use_tls {
        client_config.set("security.protocol", "ssl");
        
        // Set certificate paths if provided
        if let Some(ca_path) = &config.ca_cert_path {
            if Path::new(ca_path).exists() {
                client_config.set("ssl.ca.location", ca_path);
            } else {
                eprintln!("Warning: CA certificate not found at {}", ca_path);
            }
        }
        
        if let Some(cert_path) = &config.client_cert_path {
            if Path::new(cert_path).exists() {
                client_config.set("ssl.certificate.location", cert_path);
            } else {
                eprintln!("Warning: Client certificate not found at {}", cert_path);
            }
        }
        
        if let Some(key_path) = &config.client_key_path {
            if Path::new(key_path).exists() {
                client_config.set("ssl.key.location", key_path);
            } else {
                eprintln!("Warning: Client key not found at {}", key_path);
            }
        }
        
        println!("Using TLS connection to {}", broker);
    } else {
        println!("Using plain connection to {}", broker);
    }
    
    client_config.create().map_err(|e| Box::new(e) as Box<dyn Error>)
}

/// Publish a proof to Kafka (legacy function for compatibility)
pub async fn publish_to_kafka(proof: ZkProof) -> Result<(), Box<dyn Error>> {
    let config = KafkaConfig::default();
    let producer = create_producer(&config)?;
    
    let payload = serde_json::to_string(&proof)?;
    
    let delivery_status = producer
        .send(
            FutureRecord::to(KAFKA_TOPIC)
                .payload(&payload)
                .key(&proof.identifier),
            Duration::from_secs(5),
        )
        .await;
    
    match delivery_status {
        Ok((partition, offset)) => {
            println!("Message sent to partition {} at offset {}", partition, offset);
            Ok(())
        }
        Err((e, _)) => Err(Box::new(e)),
    }
}

/// Publish JSON value to Kafka with configuration
pub async fn publish_json_to_kafka_with_config(
    json_value: Value, 
    config: &KafkaConfig
) -> Result<(), Box<dyn Error>> {
    let producer = create_producer(config)?;
    
    let payload = json_value.to_string();
    
    // Extract identifier from JSON for the key
    let key = json_value
        .get("identifier")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    
    let delivery_status = producer
        .send(
            FutureRecord::to(KAFKA_TOPIC)
                .payload(&payload)
                .key(&key),
            Duration::from_secs(5),
        )
        .await;
    
    match delivery_status {
        Ok((partition, offset)) => {
            println!("Message sent to partition {} at offset {}", partition, offset);
            Ok(())
        }
        Err((e, _)) => Err(Box::new(e)),
    }
}

/// Publish JSON value to Kafka (uses default TLS configuration)
pub async fn publish_json_to_kafka(json_value: Value) -> Result<(), Box<dyn Error>> {
    let config = KafkaConfig::default();
    publish_json_to_kafka_with_config(json_value, &config).await
}