CREATE TABLE IF NOT EXISTS blocks (
    number BIGINT PRIMARY KEY,
    timestamp BIGINT NOT NULL,
    transaction_count INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks(timestamp);
CREATE INDEX IF NOT EXISTS idx_blocks_created_at ON blocks(created_at);

CREATE TABLE IF NOT EXISTS transactions (
    hash VARCHAR(66) PRIMARY KEY,
    block_number BIGINT NOT NULL REFERENCES blocks(number) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_transactions_block ON transactions(block_number);
CREATE INDEX IF NOT EXISTS idx_transactions_position ON transactions(block_number, position);

CREATE TABLE IF NOT EXISTS ingestion_state (
    id INTEGER PRIMARY KEY DEFAULT 1,
    last_processed_block BIGINT NOT NULL DEFAULT 0,
    mode VARCHAR(10) NOT NULL DEFAULT 'live',
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT ensure_single_row CHECK (id = 1)
);

INSERT INTO ingestion_state (id, last_processed_block, mode) 
VALUES (1, 0, 'reindex') 
ON CONFLICT (id) DO NOTHING;