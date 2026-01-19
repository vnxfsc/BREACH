# BREACH Backend API

High-performance Rust backend for the BREACH Titan Hunter game.

## Tech Stack

- **Framework**: Axum 0.7
- **Runtime**: Tokio
- **Database**: PostgreSQL 15+ with PostGIS 3.3
- **Cache**: Redis 7+
- **Authentication**: JWT + Ed25519 wallet signatures
- **Language**: Rust 2021

## Features

- 🔐 **Authentication** - Solana wallet signature verification + JWT
- 🗺️ **Map System** - PostGIS spatial queries for nearby Titans
- ⚔️ **PvP System** - ELO matchmaking, turn-based battles, seasonal ranks
- 👥 **Social** - Friends, guilds, notifications
- 🎯 **Quests** - Daily challenges with rewards
- 🏆 **Achievements** - 22 achievements across 6 categories
- 📊 **Leaderboards** - Multiple ranking types
- 📦 **Inventory** - Titan collection management
- 🔄 **Real-time** - WebSocket for live updates

## Project Structure

```
backend/
├── src/
│   ├── main.rs              # Entry point + server setup
│   ├── api/                  # HTTP endpoints (17 modules)
│   │   ├── auth.rs          # Authentication
│   │   ├── capture.rs       # Capture authorization
│   │   ├── map.rs           # Map/Titan queries
│   │   ├── player.rs        # Player management
│   │   ├── pvp.rs           # PvP matchmaking
│   │   ├── friend.rs        # Friend system
│   │   ├── guild.rs         # Guild management
│   │   ├── notification.rs  # Notifications
│   │   ├── quest.rs         # Daily quests
│   │   ├── achievement.rs   # Achievements
│   │   ├── battle.rs        # Wild battles
│   │   ├── inventory.rs     # Titan inventory
│   │   ├── leaderboard.rs   # Rankings
│   │   └── health.rs        # Health checks
│   ├── config/              # Configuration
│   ├── db/                  # Database connections
│   ├── error/               # Error handling
│   ├── middleware/          # Auth middleware
│   ├── models/              # Data models (12 modules)
│   ├── services/            # Business logic (17 modules)
│   ├── scheduler/           # Background tasks
│   ├── utils/               # Helpers (geo, etc.)
│   └── websocket/           # Real-time updates
├── migrations/              # SQL migrations (4 files)
├── config/                  # Config files
├── Dockerfile               # Container build
├── docker-compose.yml       # Local development
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
| GET | `/api/v1/player/me/stats` | Get player stats |
| GET | `/api/v1/player/:id` | Get player by ID |

### PvP Matchmaking

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/pvp/season` | Get current season |
| GET | `/api/v1/pvp/stats` | Get PvP stats |
| POST | `/api/v1/pvp/queue` | Join matchmaking queue |
| GET | `/api/v1/pvp/queue` | Get queue status |
| DELETE | `/api/v1/pvp/queue` | Leave queue |
| GET | `/api/v1/pvp/match/:id` | Get match state |
| POST | `/api/v1/pvp/match/:id/titan` | Select Titan |
| POST | `/api/v1/pvp/match/:id/surrender` | Surrender |
| POST | `/api/v1/pvp/action` | Submit action |
| GET | `/api/v1/pvp/leaderboard` | PvP rankings |
| GET | `/api/v1/pvp/history` | Match history |

### Friends

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/friends` | List friends |
| POST | `/api/v1/friends/request` | Send friend request |
| GET | `/api/v1/friends/requests` | List pending requests |
| POST | `/api/v1/friends/accept/:id` | Accept request |
| POST | `/api/v1/friends/reject/:id` | Reject request |
| DELETE | `/api/v1/friends/:id` | Remove friend |
| POST | `/api/v1/friends/:id/gift` | Send gift |
| GET | `/api/v1/friends/gifts` | List pending gifts |
| POST | `/api/v1/friends/gifts/:id/open` | Open gift |

### Guild

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/guild` | Create guild |
| GET | `/api/v1/guild/me` | Get my guild membership |
| DELETE | `/api/v1/guild/me` | Leave guild |
| GET | `/api/v1/guilds` | Search guilds |
| GET | `/api/v1/guild/:id` | Get guild details |
| PUT | `/api/v1/guild/:id` | Update guild settings |
| GET | `/api/v1/guild/:id/members` | List members |
| POST | `/api/v1/guild/:id/join` | Request to join |
| POST | `/api/v1/guild/:id/accept/:player` | Accept join request |
| POST | `/api/v1/guild/:id/reject/:player` | Reject join request |
| DELETE | `/api/v1/guild/:id/kick/:player` | Kick member |
| PUT | `/api/v1/guild/:id/role/:player` | Change role |

### Notifications

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/notifications` | List notifications |
| GET | `/api/v1/notifications/count` | Get unread count |
| PUT | `/api/v1/notifications/:id/read` | Mark as read |
| PUT | `/api/v1/notifications/read-all` | Mark all as read |
| DELETE | `/api/v1/notifications/read` | Delete read |

### Quests

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/quests` | List active quests |
| POST | `/api/v1/quests/:id/claim` | Claim reward |

### Achievements

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/achievements` | List all achievements |
| GET | `/api/v1/achievements/summary` | Summary by category |
| GET | `/api/v1/achievements/recent` | Recent unlocks |

### Battles

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/battles/wild` | Start wild battle |
| POST | `/api/v1/battles/:id/action` | Submit action |
| POST | `/api/v1/battles/:id/end` | End battle |
| GET | `/api/v1/battles/history` | Battle history |

### Inventory

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/inventory` | List my Titans |
| GET | `/api/v1/inventory/summary` | Inventory stats |
| GET | `/api/v1/inventory/favorites` | Favorite Titans |
| GET | `/api/v1/inventory/:id` | Titan details |
| PUT | `/api/v1/inventory/:id` | Update Titan |

### Leaderboards

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/leaderboard` | Get leaderboard |
| GET | `/api/v1/leaderboard/me` | My rankings |
| GET | `/api/v1/leaderboard/top` | Top by stat |

### WebSocket

| Endpoint | Description |
|----------|-------------|
| `/ws?geohash=xxx` | Real-time updates |

**WebSocket Events:**
- `TitanSpawn` - New Titan spawned
- `TitanCaptured` - Titan captured
- `TitanExpired` - Titan despawned
- `Subscribe` / `Unsubscribe` - Region subscription

## Configuration

Configuration is loaded from (in order):
1. `config/default.toml`
2. `config/local.toml` (optional)
3. Environment variables (prefix: `BREACH__`)

## Database Schema

See `migrations/` for the full schema. 36 tables including:

**Core Tables:**
- `players` - Player accounts and stats
- `titan_spawns` - Active Titan spawns
- `pois` - Points of Interest
- `player_locations` - Location history
- `capture_records` - Capture analytics
- `battle_records` - Battle history

**Social Tables:**
- `friend_requests` - Pending friend requests
- `friendships` - Established friendships
- `friend_gifts` - Gift transactions
- `guilds` - Guild information
- `guild_members` - Guild membership
- `guild_requests` - Join requests
- `notifications` - Player notifications

**PvP Tables:**
- `pvp_seasons` - Season definitions
- `player_pvp_stats` - Player ELO and stats
- `pvp_matches` - Match records
- `pvp_battle_turns` - Turn-by-turn actions
- `matchmaking_queue` - Queue entries

**Progression Tables:**
- `quest_templates` - Quest definitions
- `player_quests` - Active quests
- `achievements` - Achievement definitions
- `player_achievements` - Unlocked achievements
- `player_titans` - Titan inventory
- `battles` - Battle records
- `leaderboard_cache` - Cached rankings

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `BREACH__SERVER__PORT` | Server port | 8080 |
| `BREACH__DATABASE__URL` | PostgreSQL URL | - |
| `BREACH__REDIS__URL` | Redis URL | - |
| `BREACH__AUTH__JWT_SECRET` | JWT signing key | - |

## License

MIT
