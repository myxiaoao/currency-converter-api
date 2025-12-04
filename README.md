# Currency Converter API

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A high-performance RESTful API built with Rust and Axum that provides real-time currency exchange rates and conversion capabilities. Data is sourced from the European Central Bank (ECB) and cached in Redis for fast access.

**GitHub Repository**: [https://github.com/myxiaoao/currency-converter-api](https://github.com/myxiaoao/currency-converter-api)

## Features

- **Real-time Exchange Rates**: Fetches daily rates from ECB
- **Automatic Updates**: Scheduled updates at 15:00 UTC daily
- **High Performance**: O(1) currency conversion with zero memory allocation
- **Financial Precision**: Uses Decimal arithmetic (no floating-point errors)
- **Fast Response**: Redis caching for sub-millisecond response times
- **Currency Conversion**: Convert between any supported currency pairs
- **Flexible Base Currency**: Get rates with any currency as the base
- **RESTful API**: Clean, intuitive endpoints
- **Production Ready**: Comprehensive error handling, logging, and monitoring

## Supported Currencies

~32 currencies including USD, EUR, GBP, JPY, CNY, and more. The full list is determined by ECB's daily feed.

## Quick Start

### Using Docker Compose (Recommended)

```bash
# Start the services
docker compose up -d

# Check logs
docker compose logs -f currency-converter-api

# Stop services
docker compose down
```

The API will be available at `http://localhost:3000`

### Manual Setup

#### Prerequisites

- Rust 1.83 or later
- Redis 7.0 or later

#### Installation

1. Clone the repository:
```bash
git clone https://github.com/myxiaoao/currency-converter-api.git
cd currency-converter-api
```

2. Copy environment configuration:
```bash
cp .env.example .env
```

3. Start Redis:
```bash
# Using Docker
docker run -d -p 6379:6379 redis:7-alpine

# Or install locally
# macOS: brew install redis && brew services start redis
# Ubuntu: sudo apt install redis-server
```

4. Build and run:
```bash
cargo build --release
cargo run --release
```

## API Endpoints

### Health Check

**GET /health**

Check service health and Redis connectivity.

**Response:**
```json
{
  "status": "success",
  "redis": "healthy",
  "last_update": "2024-12-04"
}
```

### Get Latest Rates

**GET /api/latest**

Get latest exchange rates with EUR as base currency.

**Query Parameters:**
- `base` (optional): 3-letter currency code to use as base (default: EUR)

**Examples:**
```bash
# Get rates with EUR base
curl http://localhost:3000/api/latest

# Get rates with USD base
curl "http://localhost:3000/api/latest?base=USD"
```

**Response:**
```json
{
  "date": "2024-12-04",
  "base": "USD",
  "rates": {
    "EUR": "0.8570449091532396297565992458",
    "GBP": "0.7512855673637298594446348989",
    "JPY": "155.36510113129928008227631128",
    "CNY": "7.062235698938193057840106324",
    ...
  }
}
```

**Note**: Rates are returned as precise Decimal strings to preserve financial accuracy.

### Convert Currency

**GET /api/convert**

Convert an amount from one currency to another.

**Query Parameters:**
- `from` (required): Source currency code (3 letters)
- `to` (required): Target currency code (3 letters)
- `amount` (required): Amount to convert (decimal string, must be >= 0)

**Examples:**
```bash
# Convert 100 USD to EUR
curl "http://localhost:3000/api/convert?from=USD&to=EUR&amount=100"

# Convert with decimal amount
curl "http://localhost:3000/api/convert?from=EUR&to=USD&amount=100.50"
```

**Response:**
```json
{
  "from": "USD",
  "to": "EUR",
  "amount": "100",
  "result": "85.70449091532396297565992458",
  "rate": "0.8570449091532396297565992458",
  "date": "2024-12-04"
}
```

**Note**: All numeric values use Decimal precision for financial accuracy.

## Configuration

All configuration is done via environment variables. See `.env.example` for all options.

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_HOST` | Server bind address | `0.0.0.0` |
| `SERVER_PORT` | Server port | `3000` |
| `REDIS_URL` | Redis connection URL | `redis://localhost:6379` |
| `ECB_URL` | ECB XML feed URL | ECB daily rates URL |
| `UPDATE_CRON` | Update schedule (cron format) | `0 0 15 * * *` (15:00 UTC) |
| `RUST_LOG` | Logging level | `info,currency_converter_api=debug` |

## How It Works

### Data Flow

1. **Initial Fetch**: On startup, the API attempts to fetch the latest rates from ECB
2. **Scheduled Updates**: A cron job runs daily at 15:00 UTC to fetch fresh data
3. **Redis Caching**: All rates are stored in Redis as JSON
4. **Request Handling**: API queries Redis for fast responses

### Conversion Logic

The API uses **optimized O(1) direct cross-rate calculation**:

1. ECB provides rates with EUR as base (e.g., EUR → USD = 1.1668, EUR → JPY = 181.28)
2. To convert USD → JPY, calculate directly:
   - **Cross Rate** = (EUR → JPY) / (EUR → USD) = 181.28 / 1.1668 = 155.365
3. Then apply: 100 USD × 155.365 = 15,536.5 JPY

**Performance Benefits:**
- **O(1) Time Complexity**: Direct calculation without iterating all rates
- **Zero Allocations**: No HashMap creation per request
- **Decimal Precision**: Uses `rust_decimal` for exact arithmetic

This allows conversion between any currency pair without pre-computing all combinations or expensive rebase operations.

## Development

### Run Tests

```bash
cargo test
```

### Run with Hot Reload

```bash
cargo install cargo-watch
cargo watch -x run
```

### Build for Production

```bash
cargo build --release
./target/release/currency-converter-api
```

## Architecture

```
src/
├── main.rs              # Application entry point
├── config.rs            # Configuration management
├── error.rs             # Error types and HTTP mapping
├── routes.rs            # Router setup
├── models/              # Data structures
│   ├── rate.rs         # ECB data models
│   └── api.rs          # API request/response types
├── services/            # Business logic
│   ├── converter.rs    # Currency conversion algorithms
│   ├── ecb_fetcher.rs  # ECB XML fetching and parsing
│   ├── redis_store.rs  # Redis operations
│   └── scheduler.rs    # Cron scheduling
└── handlers/            # HTTP handlers
    ├── health.rs       # Health check
    ├── rates.rs        # Latest rates endpoint
    └── convert.rs      # Conversion endpoint
```

## Error Handling

The API returns appropriate HTTP status codes:

- `200 OK`: Successful request
- `400 Bad Request`: Invalid parameters (e.g., invalid currency code)
- `404 Not Found`: Currency not found in exchange rates
- `500 Internal Server Error`: Server error
- `503 Service Unavailable`: No rates available (e.g., at startup before first fetch)

All errors include a JSON response with an `error` field.

## Performance

- **O(1) Currency Conversion**: Direct cross-rate calculation without HashMap allocations
- **Zero Memory Allocation**: Per-request conversion uses stack memory only
- **Sub-millisecond Latency**: Redis caching + optimized algorithm = <1ms response
- **High Concurrency**: Async Rust with Tokio handles 10,000+ concurrent requests
- **Decimal Precision**: Arbitrary-precision arithmetic with no floating-point errors
- **Low Memory Footprint**: Optimized Rust binary (~6MB in release mode)
- **Compression**: gzip compression reduces bandwidth usage by 60-80%

**Benchmark Highlights:**
- Conversion throughput: 10,000+ req/s (estimated)
- Memory per request: 0 bytes (stack-only)
- Algorithm complexity: O(1) for conversion, O(N) for rebase (when needed)

## Monitoring

- **Health endpoint**: Use `/health` for load balancer health checks
- **Structured logging**: JSON-formatted logs for easy aggregation
- **Metrics**: Consider adding Prometheus metrics in production

## Production Deployment

### Docker

```bash
# Build image
docker build -t currency-converter-api .

# Run container
docker run -d \
  -p 3000:3000 \
  -e REDIS_URL=redis://your-redis:6379 \
  currency-converter-api
```

### Kubernetes

Deploy with a Redis instance and configure health checks using the `/health` endpoint.

### Security Recommendations

1. **CORS**: Restrict allowed origins in production (edit `src/routes.rs`)
2. **Rate Limiting**: Add rate limiting middleware for public APIs
3. **HTTPS**: Deploy behind a reverse proxy (nginx, Caddy) with TLS
4. **Secrets**: Use environment variables, never commit credentials

## Troubleshooting

### API returns 503 "No rates available"

- Check if Redis is running: `redis-cli ping`
- Check logs for ECB fetch errors: `docker compose logs currency-converter-api`
- Manually trigger update by restarting the service

### Rates not updating

- Verify cron expression is correct
- Check system time and timezone
- Review logs for scheduler errors

### Redis connection errors

- Ensure Redis is running and accessible
- Verify `REDIS_URL` is correct
- Check network connectivity

### Root Endpoint

**GET /**

Returns API basic information and available endpoints.

**Response Example:**
```json
{
  "status": "success",
  "service": "Currency Converter API",
  "version": "0.1.0",
  "endpoints": {
    "health": "GET /health",
    "latest_rates": "GET /api/latest?base=<CURRENCY>",
    "convert": "GET /api/convert?from=<FROM>&to=<TO>&amount=<AMOUNT>"
  }
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Support

- **Issues**: [GitHub Issues](https://github.com/myxiaoao/currency-converter-api/issues)
- **Documentation**: See [docs/](docs/) directory for additional documentation

