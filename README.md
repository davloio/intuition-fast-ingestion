# Intuition Fast Ingestion Service

A high-performance blockchain indexer for Intuition Network that automatically switches between reindex and live modes for optimal data ingestion.

## Features

- **Auto Mode Detection**: Automatically detects whether to run in reindex or live mode based on blockchain height and database state
- **Batch Processing**: Fetches 1000 blocks per batch in reindex mode for maximum throughput
- **Live Streaming**: WebSocket subscription for real-time block ingestion
- **PostgreSQL Storage**: Optimized batch inserts with proper indexing
- **Minimal Data Model**: Stores only essential block data (number, timestamp, transaction count) and transaction hashes

## Architecture

```
┌─────────────────────────────────────────────┐
│           Fast Ingestion Service            │
├─────────────────────────────────────────────┤
│  ┌─────────────┐      ┌──────────────┐     │
│  │ Mode Detect │──────│ RPC Client   │     │
│  └─────────────┘      └──────────────┘     │
│         │                     │             │
│         ▼                     ▼             │
│  ┌─────────────┐      ┌──────────────┐     │
│  │  Reindex    │      │  Live Mode   │     │
│  │ (1000/batch)│      │  (1 block)   │     │
│  └─────────────┘      └──────────────┘     │
│         │                     │             │
│         └──────────┬──────────┘             │
│                    ▼                        │
│           ┌──────────────┐                  │
│           │ Batch Buffer │                  │
│           └──────────────┘                  │
│                    │                        │
│                    ▼                        │
│           ┌──────────────┐                  │
│           │  PostgreSQL  │                  │
│           │ Batch Insert │                  │
│           └──────────────┘                  │
└─────────────────────────────────────────────┘
```

## Setup

1. **Environment Configuration**:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

2. **Database Setup**:
   ```bash
   # Create PostgreSQL database
   createdb intuition_indexer
   ```

3. **Run the Service**:
   ```bash
   cargo run --release
   ```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `postgresql://postgres:password@localhost:5432/intuition_indexer` | PostgreSQL connection string |
| `RPC_HTTP_URL` | `https://rpc.intuition.systems` | HTTP RPC endpoint |
| `RPC_WS_URL` | `wss://rpc.intuition.systems` | WebSocket RPC endpoint |
| `BATCH_SIZE` | `1000` | Number of blocks to fetch per batch in reindex mode |
| `MAX_CONCURRENT_REQUESTS` | `10` | Maximum concurrent RPC requests |
| `DB_MAX_CONNECTIONS` | `20` | Maximum database connections |
| `LOG_LEVEL` | `info` | Logging level |

## Mode Logic

### Auto Mode Detection
1. **Startup**: Query database for last processed block
2. **Check Gap**: Compare with chain height
3. **Reindex Mode** (gap >= 1000):
   - Batch fetch 1000 blocks via HTTP RPC
   - Continue until gap < 1000
4. **Catch-up Mode** (gap < 1000):
   - Fetch remaining blocks individually
   - Get to chain head
5. **Live Mode** (caught up):
   - Switch to WebSocket connection
   - Process blocks as they arrive in real-time

## Database Schema

```sql
-- Blocks table (minimal storage)
CREATE TABLE blocks (
    number BIGINT PRIMARY KEY,
    timestamp BIGINT NOT NULL,
    transaction_count INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Transactions table (just hashes)
CREATE TABLE transactions (
    hash VARCHAR(66) PRIMARY KEY,
    block_number BIGINT NOT NULL REFERENCES blocks(number),
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Service state tracking
CREATE TABLE ingestion_state (
    id INTEGER PRIMARY KEY DEFAULT 1,
    last_processed_block BIGINT NOT NULL,
    mode VARCHAR(10) NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

## Performance Optimizations

- **Async/Await**: Full async implementation with tokio
- **Concurrent RPC**: Parallel block fetching during reindex
- **Batch SQL Inserts**: Minimize database roundtrips
- **Connection Pooling**: Optimized database connections
- **Proper Indexing**: Fast queries on block numbers and timestamps

## Monitoring

The service provides structured logging with configurable levels:

```bash
# Set log level
export LOG_LEVEL=debug

# Run with detailed logging
cargo run --release
```

## Building for Production

```bash
# Optimized release build
cargo build --release

# The binary will be available at:
./target/release/intuition-fast-ingestion
```