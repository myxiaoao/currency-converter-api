# Running Without Docker

## Prerequisites

1. **Rust Environment**
   ```bash
   rustc --version  # Should be 1.83+
   ```

2. **Redis Service**
   ```bash
   redis-cli ping  # Should return PONG
   ```
   
   If Redis is not running:
   ```bash
   # macOS
   brew services start redis
   
   # Linux
   sudo systemctl start redis
   
   # Or run Redis with Docker
   docker run -d -p 6379:6379 redis:7-alpine
   ```

## Starting the Service

### Method 1: Foreground (Recommended for Development)
```bash
cd currency-converter-api
cargo run --release
```

### Method 2: Background
```bash
cd currency-converter-api
nohup cargo run --release > /tmp/currency-api.log 2>&1 &
echo $! > /tmp/currency-api.pid
```

### Method 3: Run Compiled Binary
```bash
cd currency-converter-api
cargo build --release
./target/release/currency-converter-api
```

## Verify Service

```bash
# Check health status
curl http://localhost:3000/health

# Expected response:
# {"status":"ok","redis":"healthy","last_update":"2025-12-03"}
```

## API Usage Examples

### 1. Health Check
```bash
curl http://localhost:3000/health | jq .
```

### 2. Get All Rates (EUR Base)
```bash
curl http://localhost:3000/api/latest | jq .
```

### 3. Get Rates with Custom Base Currency
```bash
# USD as base
curl "http://localhost:3000/api/latest?base=USD" | jq .

# CNY as base
curl "http://localhost:3000/api/latest?base=CNY" | jq .
```

### 4. Currency Conversion
```bash
# Convert 100 USD to EUR
curl "http://localhost:3000/api/convert?from=USD&to=EUR&amount=100" | jq .

# Convert 1000 CNY to USD
curl "http://localhost:3000/api/convert?from=CNY&to=USD&amount=1000" | jq .

# Convert 500 GBP to JPY
curl "http://localhost:3000/api/convert?from=GBP&to=JPY&amount=500" | jq .
```

## Service Management

### View Process
```bash
ps aux | grep currency-converter-api
```

### View Logs (Background Mode)
```bash
tail -f /tmp/currency-api.log
```

### Stop Service
```bash
# If running in foreground, press Ctrl+C

# If running in background
kill $(cat /tmp/currency-api.pid)

# Or find and stop the process directly
pkill -f currency-converter-api
```

### Restart Service
```bash
# Stop
pkill -f currency-converter-api

# Wait 2 seconds
sleep 2

# Start
cd currency-converter-api
nohup cargo run --release > /tmp/currency-api.log 2>&1 &
echo $! > /tmp/currency-api.pid
```

## Environment Variable Configuration

Customize configuration via environment variables:

```bash
# Change port
SERVER_PORT=8080 cargo run --release

# Use different Redis
REDIS_URL=redis://remote-redis:6379 cargo run --release

# Change log level
RUST_LOG=debug cargo run --release

# Combine multiple variables
SERVER_PORT=8080 REDIS_URL=redis://localhost:6379 RUST_LOG=info cargo run --release
```

## Development Mode

### Auto-reload (Requires cargo-watch)
```bash
# Install cargo-watch
cargo install cargo-watch

# Run with auto-reload
cargo watch -x run
```

### Run Tests
```bash
cargo test
```

### Code Checks
```bash
cargo check
cargo clippy
```

## Performance Optimization

### Release Mode Build
```bash
cargo build --release
```
- Binary size: 6.0MB
- Startup time: < 1 second
- Response time: < 1ms (Redis cache)

### Monitor Resource Usage
```bash
# CPU and memory usage
ps aux | grep currency-converter-api

# Detailed information
top -pid $(pgrep currency-converter-api)
```

## Troubleshooting

### Issue 1: Port Already in Use
```bash
# Error: Address already in use

# Solution: Stop the process using the port
lsof -i :3000
kill <PID>
```

### Issue 2: Redis Connection Failed
```bash
# Error: Redis connection error

# Check if Redis is running
redis-cli ping

# Start Redis
brew services start redis  # macOS
sudo systemctl start redis  # Linux
```

### Issue 3: Compilation Error
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

### Issue 4: 503 No Rates Available
```bash
# Wait for initial data loading (usually 1-2 seconds)
# Or check logs
tail -f /tmp/currency-api.log
```

## Current Running Status

**Service URL**: http://localhost:3000
**Process PID**: Check with `pgrep currency-converter-api`
**Log File**: /tmp/currency-api.log

**Management Commands**:
- Stop: `kill $(pgrep currency-converter-api)`
- Logs: `tail -f /tmp/currency-api.log`
- Status: `curl http://localhost:3000/health`

## Supported Currencies

The API supports approximately 31 currencies, including:
- USD (US Dollar)
- EUR (Euro)
- GBP (British Pound)
- JPY (Japanese Yen)
- CNY (Chinese Yuan)
- AUD, CAD, CHF, HKD, SGD, etc.

Full list available via API:
```bash
curl http://localhost:3000/api/latest | jq '.rates | keys'
```

## Automatic Updates

The service automatically fetches the latest exchange rates from ECB daily at 15:00 UTC. No manual intervention required.

---

**Note**: For production environments, Docker deployment is recommended for better isolation and portability.
