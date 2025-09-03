use crate::types::ZkProof;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde_json::Value;
use std::error::Error;
use std::time::Duration;

const KAFKA_BROKER: &str = "b-1.test.7alql0.c5.kafka.us-east-1.amazonaws.com:9092";
const KAFKA_TOPIC: &str = "twine.solana.proofs";

/// Publish a proof to Kafka
pub async fn publish_to_kafka(proof: ZkProof) -> Result<(), Box<dyn Error>> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", KAFKA_BROKER)
        .set("message.timeout.ms", "5000")
        .create()?;
    
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

/// Publish JSON value to Kafka
pub async fn publish_json_to_kafka(json_value: Value) -> Result<(), Box<dyn Error>> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", KAFKA_BROKER)
        .set("message.timeout.ms", "5000")
        .create()?;
    
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