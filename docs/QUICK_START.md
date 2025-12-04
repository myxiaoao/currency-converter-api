# Currency Converter API

## Prerequisites
- Docker & Docker Compose (recommended), OR
- Rust 1.83+ and Redis 7.0+

## Option 1: Using Docker Compose (Recommended)

### Start the API
```bash
docker compose up -d
```

### Check logs
```bash
docker compose logs -f currency-converter-api
```

### Stop the API
```bash
docker compose down
```

## Option 2: Manual Setup

### 1. Start Redis
```bash
docker run -d -p 6379:6379 redis:7-alpine
```

### 2. Configure environment
```bash
cp .env.example .env
# Edit .env if needed
```

### 3. Run the API
```bash
# Development
cargo run

# Production
cargo build --release
./target/release/currency-converter-api
```

## Testing the API

### Health Check
```bash
curl http://localhost:3000/health
```

Expected response:
```json
{
  "status": "ok",
  "redis": "healthy",
  "last_update": "2025-12-04"
}
```

### Get All Rates (EUR base)
```bash
curl http://localhost:3000/api/latest
```

### Get Rates with USD base
```bash
curl "http://localhost:3000/api/latest?base=USD"
```

### Convert Currency
```bash
# Convert 100 USD to EUR
curl "http://localhost:3000/api/convert?from=USD&to=EUR&amount=100"
```

Expected response:
```json
{
  "from": "USD",
  "to": "EUR",
  "amount": 100.0,
  "result": 95.24,
  "rate": 0.9524,
  "date": "2025-12-04"
}
```

### More Examples
```bash
# Convert 1000 JPY to USD
curl "http://localhost:3000/api/convert?from=JPY&to=USD&amount=1000"

# Get rates with GBP as base
curl "http://localhost:3000/api/latest?base=GBP"

# Convert 50 EUR to GBP
curl "http://localhost:3000/api/convert?from=EUR&to=GBP&amount=50"
```

## Troubleshooting

### API returns 503 "No rates available"
The API hasn't fetched rates yet. This happens on first startup before the initial ECB fetch completes.

**Solution:** Wait a few seconds and retry, or check logs:
```bash
docker compose logs currency-converter-api
```

### Connection refused
**Check if services are running:**
```bash
docker compose ps
```

**Restart services:**
```bash
docker compose restart
```

### Redis connection error
**Check Redis is running:**
```bash
docker compose logs redis
redis-cli ping  # Should return PONG
```

## Configuration

Edit `.env` file or set environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_PORT` | 3000 | API port |
| `REDIS_URL` | redis://localhost:6379 | Redis connection |
| `UPDATE_CRON` | 0 0 15 * * * | Update schedule (15:00 UTC) |
| `RUST_LOG` | info,currency_converter_api=debug | Log level |

## Development

### Run tests
```bash
cargo test
```

### Run with auto-reload
```bash
cargo install cargo-watch
cargo watch -x run
```

### Check code
```bash
cargo check
cargo clippy
```

## Production Deployment

### Build optimized binary
```bash
cargo build --release
```

Binary location: `target/release/currency-converter-api` (6.0MB)

### Build Docker image
```bash
docker build -t currency-converter-api:latest .
```

### Run Docker container
```bash
docker run -d \
  -p 3000:3000 \
  -e REDIS_URL=redis://your-redis:6379 \
  currency-converter-api:latest
```

## API Reference

### GET /health
Returns service health status

**Response:** 200 OK
```json
{
  "status": "ok",
  "redis": "healthy",
  "last_update": "2025-12-04"
}
```

### GET /api/latest?base={currency}
Get latest exchange rates

**Parameters:**
- `base` (optional): 3-letter currency code (default: EUR)

**Response:** 200 OK
```json
{
  "date": "2025-12-04",
  "base": "USD",
  "rates": {
    "EUR": 0.9524,
    "GBP": 0.7952,
    ...
  }
}
```

**Errors:**
- 404: Currency not found
- 503: No rates available

### GET /api/convert?from={from}&to={to}&amount={amount}
Convert currency

**Parameters:**
- `from` (required): Source currency (3 letters)
- `to` (required): Target currency (3 letters)
- `amount` (required): Amount to convert (>= 0)

**Response:** 200 OK
```json
{
  "from": "USD",
  "to": "EUR",
  "amount": 100.0,
  "result": 95.24,
  "rate": 0.9524,
  "date": "2025-12-04"
}
```

**Errors:**
- 400: Invalid parameters
- 404: Currency not found
- 503: No rates available

## Performance

- **Response time**: < 1ms (Redis cached)
- **Concurrent requests**: Handles thousands with Tokio async runtime
- **Memory footprint**: ~10-20MB
- **Binary size**: 6.0MB (optimized with LTO and strip)

## Support

For issues, check:
1. Logs: `docker compose logs -f`
2. Redis: `redis-cli ping`
3. Network: `curl http://localhost:3000/health`

Enjoy your currency API! 🚀
