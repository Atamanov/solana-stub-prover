#!/bin/bash

# Script to check and create Kafka topic if needed

TOPIC="twine.solana.proofs"
BROKER="b-1.test.7alql0.c5.kafka.us-east-1.amazonaws.com:9092"

echo "🔍 Checking Kafka topic: $TOPIC"
echo "📍 Broker: $BROKER"
echo ""

# Check if kafka tools are installed
if ! command -v kafka-topics &> /dev/null && ! command -v kafka-topics.sh &> /dev/null; then
    echo "❌ Kafka tools not found. Installing kafka..."
    echo ""
    echo "On macOS, you can install with:"
    echo "  brew install kafka"
    echo ""
    echo "Or download from: https://kafka.apache.org/downloads"
    exit 1
fi

# Determine the kafka-topics command
if command -v kafka-topics &> /dev/null; then
    KAFKA_TOPICS="kafka-topics"
elif command -v kafka-topics.sh &> /dev/null; then
    KAFKA_TOPICS="kafka-topics.sh"
else
    echo "❌ Cannot find kafka-topics command"
    exit 1
fi

# List existing topics
echo "📋 Listing existing topics..."
$KAFKA_TOPICS --list --bootstrap-server $BROKER 2>/dev/null

if [ $? -ne 0 ]; then
    echo ""
    echo "⚠️  Could not list topics. The broker might require authentication or be unreachable."
    echo ""
    echo "Try using the AWS MSK tools or AWS CLI if this is an MSK cluster:"
    echo "  aws kafka list-topics --cluster-arn <your-cluster-arn>"
    exit 1
fi

# Check if our topic exists
echo ""
echo "🔍 Checking if topic '$TOPIC' exists..."
$KAFKA_TOPICS --list --bootstrap-server $BROKER 2>/dev/null | grep -q "^$TOPIC$"

if [ $? -eq 0 ]; then
    echo "✅ Topic '$TOPIC' already exists"
    
    # Get topic details
    echo ""
    echo "📊 Topic details:"
    $KAFKA_TOPICS --describe --topic $TOPIC --bootstrap-server $BROKER 2>/dev/null
else
    echo "❌ Topic '$TOPIC' does not exist"
    echo ""
    echo "Would you like to create it? (y/n)"
    read -r response
    
    if [[ "$response" == "y" || "$response" == "Y" ]]; then
        echo "Creating topic '$TOPIC'..."
        $KAFKA_TOPICS --create \
            --topic $TOPIC \
            --bootstrap-server $BROKER \
            --partitions 3 \
            --replication-factor 1 \
            2>/dev/null
        
        if [ $? -eq 0 ]; then
            echo "✅ Topic created successfully"
        else
            echo "❌ Failed to create topic. You might need admin permissions."
            echo ""
            echo "For AWS MSK, use:"
            echo "  aws kafka create-topic --cluster-arn <arn> --topic-name $TOPIC --partitions 3"
        fi
    fi
fi