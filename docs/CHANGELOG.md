# Currency Converter API

## [0.1.0] - 2025-12-04

### Added
- Initial release of Currency Converter API
- RESTful API with Axum 0.8 framework
- ECB (European Central Bank) data source integration
- Automatic daily updates at 15:00 UTC via cron scheduler
- Redis caching for high-performance rate queries
- Support for ~32 currencies
- Three API endpoints:
  - `GET /health` - Health check with Redis status
  - `GET /api/latest?base=<CURRENCY>` - Get latest rates with optional base currency
  - `GET /api/convert?from=<FROM>&to=<TO>&amount=<AMOUNT>` - Currency conversion
- Cross-currency conversion using rebase algorithm
- Comprehensive error handling with appropriate HTTP status codes
- Structured logging with tracing
- Docker and docker compose support
- Multi-stage Dockerfile for optimized image size
- Unit tests for core conversion logic
- Production-ready configuration via environment variables

### Technical Details
- Rust 2024 edition
- Async runtime: Tokio 1.42
- Web framework: Axum 0.8
- Redis client: redis 0.27
- HTTP client: reqwest 0.12
- XML parsing: quick-xml 0.37
- Scheduler: tokio-cron-scheduler 0.14
- Optimized release build: 6.0MB binary with LTO and strip

### Dependencies (Latest as of 2025)
- axum = "0.8"
- tokio = "1.42"
- tower-http = "0.6"
- reqwest = "0.12"
- quick-xml = "0.37"
- redis = "0.27"
- tokio-cron-scheduler = "0.14"
- thiserror = "2.0"
- validator = "0.19"
- chrono = "0.4"
- serde = "1.0"
- tracing = "0.1"
