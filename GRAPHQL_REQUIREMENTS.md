# Intuition GraphQL API Requirements

## Project Overview
We're building a blockchain indexing system for Intuition Network with multiple services:

1. **intuition-fast-ingestion** (this repo) - Fast block/transaction ingestion
2. **intuition-deep-analysis** (future) - Deep blockchain analysis  
3. **intuition-graphql-api** (to build) - GraphQL API for frontend

## Database Schema
The fast ingestion service populates these PostgreSQL tables:

```sql
-- Blocks table (minimal data)
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

-- Service state (internal use)
CREATE TABLE ingestion_state (
    id INTEGER PRIMARY KEY DEFAULT 1,
    last_processed_block BIGINT NOT NULL,
    mode VARCHAR(10) NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

## Required GraphQL Queries

### 1. Homepage Stats
```graphql
query HomepageStats {
  stats {
    currentBlockHeight
    totalTransactions
  }
}
```

**Response:**
- `currentBlockHeight`: Latest block number in database
- `totalTransactions`: Total count of all transactions

### 2. Blocks List (Paginated)
```graphql
query Blocks($limit: Int, $offset: Int) {
  blocks(limit: $limit, offset: $offset) {
    items {
      number
      timestamp
      transactionCount
      createdAt
    }
    totalCount
  }
}
```

**Features:**
- Pagination with `limit` (default: 20, max: 100) and `offset`
- Return total count for pagination UI
- Order by block number DESC (newest first)

### 3. Transactions List (Paginated)
```graphql
query Transactions($limit: Int, $offset: Int, $blockNumber: BigInt) {
  transactions(limit: $limit, offset: $offset, blockNumber: $blockNumber) {
    items {
      hash
      blockNumber
      position
      createdAt
    }
    totalCount
  }
}
```

**Features:**
- Pagination with `limit` (default: 20, max: 100) and `offset`
- Optional filter by `blockNumber`
- Return total count for pagination UI
- Order by block number DESC, then position ASC

## Technical Requirements

- **Language**: Rust
- **GraphQL Framework**: async-graphql (recommended)
- **Database**: PostgreSQL (same as fast ingestion)
- **Connection**: Read-only access to existing database
- **Performance**: Use connection pooling (sqlx recommended)

## Database Connection
```
DATABASE_URL=postgresql://jumpiix@localhost:5432/intuition_indexer
```

## Architecture Notes
- This is a **read-only** service
- Should NOT interfere with fast ingestion performance
- Keep queries simple and efficient
- Consider adding basic query caching later
- No mutations needed - data comes from ingestion service

## Repository Name
Create as: `intuition-graphql-api`

## Additional Notes
- Start simple - just these 3 queries
- Focus on performance and clean code
- Use proper error handling
- Add basic logging
- Keep schema extensible for future features

This is the foundation - we'll add more complex queries and features later as needed.