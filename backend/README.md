# BREACH Backend API

High-performance Rust backend for the BREACH Titan Hunter game.

## Tech Stack

- **Framework**: Axum 0.7
- **Runtime**: Tokio
- **Database**: PostgreSQL + PostGIS
- **Cache**: Redis
- **Language**: Rust 2021

## Project Structure

```
backend/
├── src/
│   ├── main.rs              # Entry point
│   ├── api/                  # HTTP endpoints
│   │   ├── auth.rs          # Authentication
│   │   ├── capture.rs       # Capture authorization
│   │   ├── map.rs           # Map/Titan queries
│   │   ├── player.rs        # Player management
│   │   └── health.rs        # Health checks
│   ├── config/              # Configuration
│   ├── db/                  # Database connections
│   ├── error/               # Error handling
│   ├── middleware/          # Auth middleware
│   ├── models/              # Data models
│   ├── services/            # Business logic
│   │   ├── auth.rs          # JWT + wallet verification
│   │   ├── capture.rs       # Capture flow
│   │   ├── location.rs      # Location verification
│   │   ├── map.rs           # Spatial queries
│   │   ├── player.rs        # Player CRUD
│   │   └── spawn.rs         # Titan spawning
│   ├── utils/               # Helpers
│   └── websocket/           # Real-time updates
├── migrations/              # SQL migrations
├── config/                  # Config files
└── Cargo.toml
```

## Quick Start

### Prerequisites

- Rust 1.75+
- PostgreSQL 15+ with PostGIS
- Redis 7+

### Setup

```bash
# Clone and navigate
cd backend

# Copy environment file
cp .env.example .env

# Edit .env with your settings
vim .env

# Create database
createdb breach
psql breach -c "CREATE EXTENSION postgis;"

# Run migrations
cargo sqlx migrate run

# Start server
cargo run
```

### Development

```bash
# Run with hot reload
cargo watch -x run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## API Endpoints

### Authentication

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/auth/challenge` | Get signing challenge |
| POST | `/api/v1/auth/authenticate` | Submit signed message |
| POST | `/api/v1/auth/refresh` | Refresh JWT token |

### Map

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/map/titans` | Get nearby Titans |
| GET | `/api/v1/map/pois` | Get POIs in bounds |
| POST | `/api/v1/map/location` | Report location |

### Capture

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/capture/request` | Request capture auth |
| POST | `/api/v1/capture/confirm` | Confirm capture |

### Player

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/player/me` | Get current player |
| PUT | `/api/v1/player/me` | Update profile |
| GET | `/api/v1/player/me/stats` | Get stats |
| GET | `/api/v1/player/:id` | Get player by ID |
| GET | `/api/v1/leaderboard` | Get leaderboard |

### WebSocket

| Endpoint | Description |
|----------|-------------|
| `/ws?geohash=xxx` | Real-time updates |

## Configuration

Configuration is loaded from (in order):
1. `config/default.toml`
2. `config/local.toml` (optional)
3. Environment variables (prefix: `BREACH__`)

## Database Schema

See `migrations/` for the full schema. Key tables:

- `players` - Player accounts
- `titan_spawns` - Active Titan spawns
- `pois` - Points of Interest
- `player_locations` - Location history
- `capture_records` - Capture analytics
- `battle_records` - Battle history

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `BREACH__SERVER__PORT` | Server port | 8080 |
| `BREACH__DATABASE__URL` | PostgreSQL URL | - |
| `BREACH__REDIS__URL` | Redis URL | - |
| `BREACH__AUTH__JWT_SECRET` | JWT signing key | - |

## License

MIT
