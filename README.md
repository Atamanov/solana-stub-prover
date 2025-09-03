# Twine Solana Stub Prover

This is a simplified stub prover for Solana that generates SP1 proofs matching the `PublicCommitments` structure of the twine-solana-prover.

## Features

- Fetches account data from Solana devnet using RPC
- Performs simple validation: `end_slot > start_slot`
- Generates SP1 proofs with matching public commitments structure
- **Generates Groth16 proofs** for on-chain verification (default)
- Supports compressed proofs for faster generation (optional)
- Saves proof to `last_proof.json` and verification key to `vkey.json`
- Publishes proofs to Kafka topic `twine.solana.proofs` as JSON
- Includes a Kafka consumer to monitor published proofs

## Building

```bash
cd /Users/alexander/Projects/Twine/solana3/solana-stub-prover
cargo build --release
```

## Prover Usage

### Execute Mode (for testing without proof generation)

```bash
RUST_LOG=info cargo run --release --bin solana-stub-prover -- \
  --start-slot 100000 \
  --end-slot 100100 \
  --account "11111111111111111111111111111111" \
  --execute
```

### Prove Mode (generates and publishes proof to Kafka)

#### Generate Groth16 Proof (default, for on-chain verification)
```bash
RUST_LOG=info cargo run --release --bin solana-stub-prover -- \
  --start-slot 100000 \
  --end-slot 100100 \
  --account "11111111111111111111111111111111" \
  --prove
```

#### Generate Compressed Proof Only (faster, not on-chain verifiable)
```bash
RUST_LOG=info cargo run --release --bin solana-stub-prover -- \
  --start-slot 100000 \
  --end-slot 100100 \
  --account "11111111111111111111111111111111" \
  --prove \
  --compressed-only
```

### Prover Parameters

#### Core Parameters
- `--start-slot`: Starting slot number
- `--end-slot`: Ending slot number (must be > start_slot)
- `--account`: Solana account pubkey in base58 format to monitor
- `--execute`: Run in execute mode (no proof generation)
- `--prove`: Generate proof and publish to Kafka
- `--groth16`: Generate Groth16 proof for on-chain verification (default: true)
- `--compressed-only`: Generate only compressed proof (faster, not verifiable on-chain)

#### Kafka Connection Parameters
- `--kafka-broker <ADDRESS>`: Override Kafka broker address
- `--kafka-tls`: Use TLS for Kafka connection (default: true)
- `--no-kafka-tls`: Disable TLS, use plain connection
- `--kafka-ca-cert <PATH>`: CA certificate file path (default: ./ca.crt)
- `--kafka-client-cert <PATH>`: Client certificate file path (default: ./user.crt)
- `--kafka-client-key <PATH>`: Client key file path (default: ./user.key)

## Kafka Consumer

A consumer application is included to listen to the `twine.solana.proofs` topic and display proof messages.

### Basic Usage

```bash
# Listen to new messages
cargo run --release --bin consumer

# Read from beginning of topic
cargo run --release --bin consumer -- --from-beginning

# Minimal output (only proof IDs)
cargo run --release --bin consumer -- --minimal

# Raw JSON output
cargo run --release --bin consumer -- --raw
```

### Connection Options

```bash
# Local Kafka instance
cargo run --release --bin consumer -- --broker localhost:9092

# With SASL authentication
cargo run --release --bin consumer -- --sasl --username myuser --password mypass

# Using environment variables
export KAFKA_USERNAME=myuser
export KAFKA_PASSWORD=mypass
cargo run --release --bin consumer -- --sasl

# Different security protocols
cargo run --release --bin consumer -- --security-protocol ssl
cargo run --release --bin consumer -- --security-protocol sasl_ssl --sasl
```

### Debugging Connection Issues

```bash
# Enable debug output
cargo run --release --bin consumer -- --debug

# Increase connection timeout
cargo run --release --bin consumer -- --connection-timeout 60
```

### Consumer Parameters

#### Connection Parameters
- `--broker <BROKER>` - Kafka broker address (default: kafka-bootstrap.twine.limited:443 with TLS)
- `--group-id <ID>` - Consumer group ID (default: solana-proof-consumer)
- `--connection-timeout <SECS>` - Connection timeout in seconds (default: 30)

#### TLS/Security Parameters
- `--tls` - Use TLS connection (default: true)
- `--no-tls` - Disable TLS, use plain connection
- `--ca-cert <PATH>` - CA certificate file path (default: ./ca.crt)
- `--client-cert <PATH>` - Client certificate file path (default: ./user.crt)
- `--client-key <PATH>` - Client key file path (default: ./user.key)
- `--security-protocol <PROTO>` - Security protocol: plaintext, ssl, sasl_plaintext, sasl_ssl

#### SASL Authentication Parameters
- `--sasl` - Enable SASL authentication
- `--username <USER>` - SASL username (or set KAFKA_USERNAME env var)
- `--password <PASS>` - SASL password (or set KAFKA_PASSWORD env var)
- `--sasl-mechanism <MECH>` - SASL mechanism: PLAIN, SCRAM-SHA-256, SCRAM-SHA-512

#### Output Options
- `--from-beginning` - Start reading from the beginning of the topic
- `--raw` - Show raw JSON output
- `--minimal` - Show only proof identifiers
- `--debug` - Enable debug output

### Example Consumer Output

#### Standard Output
```
╔══════════════════════════════════════════════════════════════════════
║ 📦 New Proof Received at 2024-01-01 12:00:00 UTC
╟──────────────────────────────────────────────────────────────────────
║ Identifier: solana-stub-290000000-290000100
║ Proof Kind: SolanaConsensusProof
║ Proof Type: SP1
║ Version: 1
║ 
║ 📊 Public Commitments:
║   Start Slot: 290000000
║   End Slot: 290000100
║   Epoch: 671
║   Monitored Accounts: 1
║   Validations Passed: true
╚══════════════════════════════════════════════════════════════════════
```

#### Minimal Output
```
[2024-01-01 12:00:00 UTC] Proof ID: solana-stub-290000000-290000100
```

## Kafka Configuration

### Default Endpoints

- **TLS/SSL (default)**: `kafka-bootstrap.twine.limited:443`
- **Plain (legacy)**: `b-1.test.7alql0.c5.kafka.us-east-1.amazonaws.com:9092`
- **Topic**: `twine.solana.proofs`
- **Message format**: JSON-serialized proof data

### TLS Certificate Setup

The prover and consumer use TLS by default. Place your certificates in the project directory:

```bash
# Required certificate files
./ca.crt        # CA certificate
./user.crt      # Client certificate  
./user.key      # Client private key
```

### Using TLS (Default)

```bash
# Prover with TLS (default)
cargo run --release --bin solana-stub-prover -- \
  --start-slot 100000 \
  --end-slot 100100 \
  --account "11111111111111111111111111111111" \
  --prove

# Consumer with TLS (default)
cargo run --release --bin consumer
```

### Using Custom Certificate Paths

```bash
# Prover with custom cert paths
cargo run --release --bin solana-stub-prover -- \
  --start-slot 100000 \
  --end-slot 100100 \
  --account "11111111111111111111111111111111" \
  --prove \
  --kafka-ca-cert /path/to/ca.crt \
  --kafka-client-cert /path/to/client.crt \
  --kafka-client-key /path/to/client.key

# Consumer with custom cert paths
cargo run --release --bin consumer -- \
  --ca-cert /path/to/ca.crt \
  --client-cert /path/to/client.crt \
  --client-key /path/to/client.key
```

### Disabling TLS (Plain Connection)

```bash
# Prover without TLS
cargo run --release --bin solana-stub-prover -- \
  --start-slot 100000 \
  --end-slot 100100 \
  --account "11111111111111111111111111111111" \
  --prove \
  --no-kafka-tls

# Consumer without TLS
cargo run --release --bin consumer -- --no-tls
```

## Environment Variables

```bash
# Set SP1 prover mode
export SP1_PROVER=local  # or network, mock

# For Kafka authentication (consumer)
export KAFKA_USERNAME=myuser
export KAFKA_PASSWORD=mypass
```

## Generated Files

When running in prove mode, the following files are created:

- **`vkey.json`** - The verification key for the proof (created once per program)
- **`last_proof.json`** - The most recent proof generated (Groth16 or compressed)
- **`last_kafka_message.json`** - The complete message sent to Kafka, including metadata

These files are useful for:
- Debugging and verification
- On-chain deployment (verification key)
- Manual proof submission or verification
- Auditing the exact data sent to Kafka

## Example Accounts (Devnet)

- System Program: `11111111111111111111111111111111`
- Token Program: `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`

## Project Structure

```
solana-stub-prover/
├── lib/               # Shared library with data structures
│   └── src/
│       └── lib.rs    # PublicCommitments and ProverInput types
├── program/           # SP1 zkVM program
│   └── src/
│       └── main.rs   # Proof validation logic
├── script/            # Main applications
│   └── src/
│       ├── bin/
│       │   ├── main.rs     # Prover application
│       │   └── consumer.rs # Kafka consumer
│       ├── lib.rs          # Module exports
│       ├── types.rs        # Shared types
│       ├── utils.rs        # Utility functions
│       ├── solana.rs       # Solana RPC functions
│       └── kafka.rs        # Kafka publishing
└── Cargo.toml         # Workspace configuration
```

## Troubleshooting

### Connection Issues

If you see errors like "BrokerTransportFailure" or "AllBrokersDown":

1. **Test network connectivity:**
   ```bash
   telnet broker.address.com 9092
   ```

2. **Check if authentication is required:**
   ```bash
   cargo run --release --bin consumer -- --sasl --username user --password pass
   ```

3. **Try a local Kafka instance:**
   ```bash
   cargo run --release --bin consumer -- --broker localhost:9092
   ```

4. **Enable debug mode for detailed errors:**
   ```bash
   cargo run --release --bin consumer -- --debug
   ```

### Slot Data Issues

- The RPC may return data from a more recent slot than requested
- Historical slot data may not be available on devnet
- The prover will use the actual slot returned and show a warning

## Notes

- This is a stub implementation that generates dummy values for:
  - ESR (validator set merkle root): all zeros
  - Total active stake: 1 billion lamports
  - Validator count: 100
  - Bank hashes: SHA256 of slot numbers
  
- The proof validates only that `end_slot > start_slot`
- Account data is fetched from Solana devnet RPC
- The consumer automatically commits offsets every second
- Consumer will stop after 10 consecutive errors