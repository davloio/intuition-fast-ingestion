# Deployment Guide

## Server Setup

### 1. Directory Structure
```bash
# On server, place in existing project structure
/path/to/projects/
├── intuition/                    # Your intuition folder
│   ├── intuition-fast-ingestion/ # This service
│   ├── other-backend/            # Your existing backend
│   └── frontend/                 # Your existing frontend
```

### 2. Database Setup
Since you already have a PostgreSQL database, just create the new database:

```bash
# Connect to your existing PostgreSQL
psql -U your_username

# Create new database for intuition indexer
CREATE DATABASE intuition_indexer;
\q
```

### 3. Environment Configuration
Create `.env` file on server:

```bash
# Database (use your existing PostgreSQL server)
DATABASE_URL=postgresql://your_username:your_password@localhost:5432/intuition_indexer

# Intuition Network RPC
RPC_HTTP_URL=https://rpc.intuition.systems
RPC_WS_URL=wss://rpc.intuition.systems

# Performance settings
BATCH_SIZE=1000
MAX_CONCURRENT_REQUESTS=10
DB_MAX_CONNECTIONS=20

# Port configuration
PORT=5555

# Logging
LOG_LEVEL=info
```

### 4. Build and Deploy
```bash
# On server in intuition folder
git clone <your-repo-url> intuition-fast-ingestion
cd intuition-fast-ingestion

# Build release version
cargo build --release

# Copy binary to system location (optional)
sudo cp target/release/intuition-fast-ingestion /usr/local/bin/
```

### 5. Systemd Service
Create `/etc/systemd/system/intuition-fast-ingestion.service`:

```ini
[Unit]
Description=Intuition Fast Ingestion Service
After=network.target postgresql.service

[Service]
Type=simple
User=your_username
WorkingDirectory=/path/to/projects/intuition/intuition-fast-ingestion
ExecStart=/usr/local/bin/intuition-fast-ingestion
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

### 6. Start Service
```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable and start service
sudo systemctl enable intuition-fast-ingestion
sudo systemctl start intuition-fast-ingestion

# Check status
sudo systemctl status intuition-fast-ingestion

# View logs
sudo journalctl -u intuition-fast-ingestion -f
```

## Port Configuration

The service now runs on **port 5555** (configurable via `PORT` environment variable).

Your services will be:
- **Port 5555**: Intuition Fast Ingestion (this service)
- **Your existing ports**: Other backend/frontend services

## Monitoring

### Check if service is running:
```bash
# Check process
ps aux | grep intuition-fast-ingestion

# Check port
netstat -tlnp | grep 5555

# View logs
tail -f /var/log/syslog | grep intuition
```

### Database monitoring:
```bash
# Connect to database
psql postgresql://your_username@localhost:5432/intuition_indexer

# Check ingestion progress
SELECT * FROM ingestion_state;
SELECT COUNT(*) FROM blocks;
SELECT COUNT(*) FROM transactions;
```

## File Structure on Server
```
/path/to/projects/intuition/intuition-fast-ingestion/
├── target/release/intuition-fast-ingestion  # Binary
├── .env                                      # Environment config
├── migrations/                              # Database migrations
└── src/                                     # Source code
```

## Notes
- Service auto-starts on server reboot
- Logs to systemd journal
- Database migrations run automatically
- Restarts automatically if it crashes
- Uses existing PostgreSQL instance
- Runs on port 5555 (configurable)