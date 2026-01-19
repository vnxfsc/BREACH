# Changelog

All notable changes to BREACH will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Planned
- Mobile app development (Flutter)
- AR capture system integration
- Mainnet deployment

---

## [0.7.3] - 2026-01-20

### Fixed - Smart Contract Security

#### Titan Transfer Ownership Validation (`titan_nft`)
- **Security Fix**: Added `owner` field to `TitanData` struct to track current owner
- **Fixed**: Transfer instruction now validates that signer is the actual owner of the Titan
- **Updated**: `TitanData` account size increased from 118 to 150 bytes (added 32-byte Pubkey)
- Mint instruction now initializes `owner` field to the initial capturer

#### Files Changed
- `contracts/programs/titan_nft/src/state/titan.rs` - Added `owner: Pubkey` field
- `contracts/programs/titan_nft/src/instructions/transfer.rs` - Added ownership verification
- `contracts/programs/titan_nft/src/instructions/mint_titan.rs` - Initialize owner on mint
- `contracts/tests/test-titan.ts` - Updated `TITAN_DATA_SIZE` constant

### Added - Backend Solana Integration

#### Solana Service (`backend/src/services/solana.rs`)
- **New**: Complete Solana RPC client service for on-chain interactions
- Backend keypair loading from JSON file
- SOL and BREACH token balance queries
- Transaction building and signing utilities
- Add experience instruction builder for titan_nft program
- Record capture/battle instruction builders for game_logic program

#### Solana API Endpoints (`backend/src/api/solana.rs`)
- `GET /api/v1/solana/backend-info` - Backend wallet and program info
- `GET /api/v1/solana/balance/:address` - SOL balance query
- `GET /api/v1/solana/breach-balance/:address` - BREACH token balance query

#### Battle & Capture Enhancements
- `POST /api/v1/battle/complete-with-rewards` - Complete battle with on-chain rewards
- `POST /api/v1/capture/confirm-with-rewards` - Confirm capture with on-chain recording
- Added `SolanaError` variant to `AppError` enum

### Deployed - Smart Contracts (Devnet)

Both programs upgraded and tested on Solana Devnet:

| Program | Address | Tests |
|---------|---------|-------|
| titan_nft | `3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7` | 22/22 ✅ |
| game_logic | `DLk2GnDu9AYn7PeLprEDHDYH9UWKENX47UqqfeiQBaSX` | 15/15 ✅ |

### Technical Details
- **titan_nft tests**: Initialize, Mint, Level Up, Evolve, Fuse, Transfer, Pause/Unpause, Authorization
- **game_logic tests**: Initialize, Record Capture, Record Battle, Add Experience, Pause/Unpause
- **Total on-chain tests**: 37 passing

---

## [0.7.2] - 2026-01-20

### Added - Unit Tests

#### Test Infrastructure
- Added library target (`lib.rs`) to enable unit testing
- Created test utilities module (`tests/common/mod.rs`)
- Added dev dependencies: `mockall`, `fake`, `pretty_assertions`, `test-case`, `async-trait`

#### Geographic Utilities Tests (`utils/geo.rs`) - 18 tests
- **Haversine distance**: Tokyo-Osaka (~400km), same point, antipodal, short distance, symmetry
- **Bearing**: North, East, South, West directions
- **Destination point**: North, East, round-trip verification
- **Random point in circle**: Bounds checking, uniform distribution
- **Geohash neighbors**: Valid, short, unique, edge cases
- **Edge cases**: Poles, date line crossing

#### Model Tests (`models/`) - 24 tests
- **Titan models** (`titan.rs`): Element enum conversion, serialization, ThreatClass validation, GeoPoint, PlayerLocation, CaptureRequest
- **Player models** (`player.rs`): PlayerSession serialization, experience/level calculations, UpdatePlayer, PlayerStats

#### Authentication Tests (`services/auth.rs`) - 15 tests
- **Challenge generation**: Format, unique nonce, 5-minute expiry
- **JWT tokens**: Generate and verify, expiry time, invalid token, wrong secret
- **Signature verification**: Invalid wallet, wrong length, invalid signature format
- **Request/Response serialization**: AuthChallenge, AuthRequest, AuthResponse

#### Integration Tests (`tests/api_tests.rs`)
- Health check endpoints
- Authentication flow tests
- Protected endpoint access tests
- Map, Quest, Achievement, Marketplace, Chat, PvP endpoint tests

### Technical Details
- **Total tests**: 57 unit tests passing
- **Test coverage**: utils, models, services
- **Dependencies added**: `futures-util = "0.3"`, `base64 = "0.21"`
- Run tests: `cargo test --lib`

---

## [0.7.1] - 2026-01-20

### Fixed - Code Quality & TODO Completion

#### Token Refresh (`api/auth.rs`)
- **Implemented** token refresh endpoint that was previously returning "not implemented"
- Uses `AuthPlayer` extractor to validate existing token
- Returns new JWT with extended expiry

#### Chat WebSocket Broadcast (`api/chat.rs`, `websocket/mod.rs`)
- **Implemented** real-time WebSocket broadcasting for chat messages
- Added `ChatMessage`, `ChatMessageEdited`, `ChatMessageDeleted` WebSocket event types
- Extended `Broadcaster` with chat-specific methods:
  - `subscribe_chat_channel()` - Subscribe player to channel
  - `unsubscribe_chat_channel()` - Unsubscribe from channel
  - `broadcast_chat_message()` - Broadcast to channel subscribers
  - `broadcast_to_player()` - Direct message to specific player
  - `is_player_online()` - Check player online status

#### Redis Online Status (`services/chat.rs`)
- **Implemented** Redis-based player online status tracking
- Added methods: `set_player_online()`, `is_player_online()`, `refresh_online_status()`, `set_player_offline()`
- Online status TTL: 5 minutes (auto-expire)
- Private chat now shows accurate online status for participants

#### Marketplace SQL Query (`services/marketplace.rs`)
- **Fixed** dynamic SQL query parameter binding
- Proper tracking of parameter indices ($2, $3, etc.)
- Only binds parameters that are actually used in query
- Prevents SQL errors with optional filter parameters

#### Data Model Fix (`models/marketplace.rs`)
- **Fixed** `TitanListingInfo.genes` type from `Option<serde_json::Value>` to `Vec<u8>`
- Matches database `BYTEA` column type

### Technical Details
- All TODO comments removed from codebase
- 0 compilation errors, 34 warnings (unused code)
- Full test coverage for all fixed endpoints

---

## [0.7.0] - 2026-01-20

### Added - Marketplace & Chat Systems

#### NFT Marketplace
- **Database Schema** (`20260120000004_marketplace.sql`)
  - `marketplace_listings` - NFT listing support (fixed price & auctions)
  - `auction_bids` - Auction bid tracking
  - `marketplace_transactions` - Transaction history
  - `price_offers` - Direct buy offers
  - `listing_favorites` - Watchlist functionality
  - `price_history` - Price analytics data
  - `marketplace_stats` - Aggregated statistics

- **Marketplace Service** (`services/marketplace.rs`)
  - Create/cancel listings (fixed price & auction)
  - Buy fixed-price listings
  - Place auction bids
  - Make/accept/reject offers
  - Favorites management
  - Transaction history
  - Price chart analytics
  - 2.5% platform fee calculation

- **Marketplace API** (15 endpoints)
  - `GET /marketplace` - Search listings
  - `POST /marketplace/listings` - Create listing
  - `GET /marketplace/listings/:id` - Get listing details
  - `DELETE /marketplace/listings/:id` - Cancel listing
  - `POST /marketplace/listings/:id/buy` - Buy listing
  - `GET /marketplace/listings/:id/bids` - Get auction bids
  - `POST /marketplace/listings/:id/bids` - Place bid
  - `POST /marketplace/offers` - Make offer
  - `GET /marketplace/offers/received` - Received offers
  - `GET /marketplace/offers/sent` - Sent offers
  - `POST /marketplace/offers/:id/accept` - Accept offer
  - `POST /marketplace/offers/:id/reject` - Reject offer
  - `GET /marketplace/favorites` - Get favorites
  - `POST/DELETE /marketplace/favorites/:id` - Manage favorites
  - `GET /marketplace/my-listings` - My listings
  - `GET /marketplace/stats` - Marketplace statistics
  - `GET /marketplace/history` - Transaction history
  - `GET /marketplace/price-chart` - Price chart data

#### Real-time Chat System
- **Database Schema** (`20260120000005_chat.sql`)
  - `chat_channels` - Channel types (world/guild/private/trade/help)
  - `chat_messages` - Message storage with replies
  - `chat_read_status` - Read receipts & mute settings
  - `chat_blocked_users` - User blocking
  - `chat_reports` - Message reporting
  - Pre-created system channels (World/Trade/Help)
  - PostgreSQL functions for private channels & unread counts

- **Chat Service** (`services/chat.rs`)
  - Multi-channel support (world/guild/private/trade/help)
  - Message send/edit/delete
  - Reply threading
  - Read status tracking
  - Channel muting
  - User blocking
  - Message reporting

- **Chat API** (13 endpoints)
  - `GET /chat/channels` - Get player channels
  - `POST /chat/channels/private` - Start private chat
  - `GET /chat/channels/:id/messages` - Get messages
  - `POST /chat/channels/:id/messages` - Send message
  - `POST /chat/channels/:id/read` - Mark as read
  - `POST /chat/channels/:id/mute` - Mute channel
  - `POST /chat/channels/:id/unmute` - Unmute channel
  - `PUT /chat/messages/:id` - Edit message
  - `DELETE /chat/messages/:id` - Delete message
  - `POST /chat/messages/:id/report` - Report message
  - `GET /chat/blocked` - Get blocked users
  - `POST /chat/blocked` - Block user
  - `DELETE /chat/blocked/:id` - Unblock user

### Technical Details
- Total API endpoints: 90+
- Database tables: 45+
- Services: 19
- Models: 15 modules

---

## [0.6.0] - 2026-01-20

### Added
- **PvP Matchmaking System** - Complete competitive multiplayer
  - ELO rating system (starting 1000, K-factor 32)
  - 7-tier rank system (Bronze → Champion) with 5 divisions each
  - Real-time matchmaking queue with ELO-based opponent matching
  - Auto-expansion of search range (±50 every 10 seconds)
  - Turn-based combat (30s per turn)
  - Action types: Attack, Special, Defend, Item
  - Win/loss streak tracking
  - Seasonal leaderboards and rewards
  - PvP match history

- **Social Features** - Complete social system
  - Friend system (send/accept/reject requests)
  - Friend list with online status
  - Daily gift sending between friends
  - Gift opening with rewards
  - Friend removal

- **Guild System** - Full guild management
  - Guild creation with name, tag, description
  - Role hierarchy (Member → Elder → Co-Leader → Leader)
  - Join requests and approvals
  - Member management (kick, promote, demote)
  - Guild search and discovery
  - Guild statistics and leaderboard

- **Notification System** - In-game notifications
  - 14 notification types (friend, guild, achievement, etc.)
  - Read/unread status tracking
  - Batch operations (mark all read, delete read)
  - Unread count API

- **Daily Quest System** - Daily challenges
  - 13 quest templates across 7 types
  - Auto-assignment of 4 daily quests
  - Progress tracking
  - Reward claiming (XP + BREACH tokens)

- **Achievement System** - Player progression rewards
  - 22 achievements across 6 categories
  - Progress tracking with unlock detection
  - Achievement summary by category
  - Recent achievements feed

- **Battle System** - Wild Titan battles
  - Turn-based combat mechanics
  - Damage calculation with RNG
  - Battle history tracking
  - XP and reward distribution

- **Inventory System** - Titan collection management
  - Player Titan listing with filters
  - Titan details and statistics
  - Favorite Titans feature
  - Inventory summary (by element, threat class)

- **Leaderboard System** - Multiple ranking types
  - Experience, Captures, Battles, BREACH earned
  - Weekly leaderboards
  - Player rank lookup
  - Top players by stat

### Database
- 4 new migration files
- 15+ new database tables
- Custom PostgreSQL types and functions
- Optimized indexes for queries

### API Endpoints
- 30+ new API endpoints
- All endpoints tested (100% pass rate)

### Bug Fixes
- Fixed route conflict (/leaderboard duplicate)
- Fixed database port configuration
- Fixed SQL type mismatch (INT4 vs INT8)

---

## [0.5.0] - 2026-01-20

### Added
- **Backend API** - Rust Axum high-performance backend
  - Authentication (wallet signature + JWT)
  - Map API (nearby Titans, POIs)
  - Capture authorization service
  - Location verification & anti-spoofing
  - Player management & leaderboard
  - WebSocket real-time updates
  - PostgreSQL + PostGIS database schema
  - Redis caching layer
  - Full configuration system

### Technical
- 20+ Rust source files
- Database migrations
- Environment configuration
- API documentation

---

## [0.4.0] - 2026-01-20

### Added
- **Game Economy Whitepaper** - Comprehensive economic design document
  - Stakeholder analysis (players, investors, project team)
  - Token flow model with dynamic emission control
  - Titan NFT economy with supply/demand balance
  - Geographic fairness algorithm
  - Player progression systems (newbie journey, long-term goals)
  - Monetization model (non-P2W)
  - Anti-cheat systems with location verification
  - Social systems (friends, guilds, world boss)
  - Emergency protocols and circuit breakers
  - Key metrics and monitoring dashboard

- **Geographic System Design** - Complete map and location system
  - POI (Point of Interest) system with 10 categories
  - Terrain types and element assignment matrix
  - Titan spawn algorithm with probability calculations
  - Location verification and anti-spoofing
  - Database schema (PostgreSQL + PostGIS)
  - Redis caching strategy for spatial queries
  - RESTful API and WebSocket event design
  - Global deployment architecture (3 regions)
  - GDPR and privacy compliance

### Documentation
- New file: `docs/GAME_ECONOMY_WHITEPAPER.md`
- New file: `docs/GEOGRAPHIC_SYSTEM_DESIGN.md`

---

## [0.3.1] - 2026-01-20

### Added
- **$BREACH Token** created on Devnet using SPL Token standard
  - Mint Address: `CSH2Vz4MbgTLzB9SYJ7gBwNsyu7nKpbvEJzKQLgmmjt4`
  - Total Supply: 1,000,000,000 BREACH
  - Decimals: 9 (same as SOL)
- Token creation script (`create-breach-token.sh`)
- Token metadata script (`add-token-metadata.ts`)

### Token Allocation
| Category | Percentage | Amount |
|----------|------------|--------|
| Play-to-Earn | 35% | 350,000,000 |
| Ecosystem | 25% | 250,000,000 |
| Team (Vested) | 15% | 150,000,000 |
| Treasury | 10% | 100,000,000 |
| Liquidity | 10% | 100,000,000 |
| Advisors | 5% | 50,000,000 |

---

## [0.3.0] - 2026-01-20

### Added

#### Game Logic Program
- **Deployed to Devnet**: `DLk2GnDu9AYn7PeLprEDHDYH9UWKENX47UqqfeiQBaSX`
- `initialize` - Initialize game configuration
- `record_capture` - Record Titan capture events with rewards
- `record_battle` - Record battle results and experience gains
- `add_experience` - Add experience to Titans
- `distribute_reward` - Distribute $BREACH token rewards
- `update_config` / `set_paused` - Admin controls

#### Account Structures (Game Logic)
- `GameConfig` (228 bytes) - Game configuration with reward settings
- `BattleRecord` (122 bytes) - Battle history records
- `CaptureRecord` (83 bytes) - Capture history records

#### Testing
- Game Logic integration test suite (15 tests, 100% passing)
- Basic functionality: Initialize, Update Config, Record Capture/Battle
- Edge cases: Expired Signature (rejected), Battle Self (rejected)
- Authorization: Invalid Backend, Unauthorized Pause
- **Total project tests: 37/37 passing** ✅

### Technical Details
- Framework: Pinocchio 0.8
- Program Size: ~29KB
- Interacts with Titan NFT Program via CPI

---

## [0.2.0] - 2026-01-20

### Added

#### Smart Contracts
- **Titan NFT Program** - Complete implementation with Pinocchio framework
  - `initialize` - Program initialization with CPI account creation
  - `mint_titan` - Mint Titan NFTs with auto Player/Titan account creation
  - `level_up` - Level up Titans (requires experience)
  - `evolve` - Evolve Titans (requires Level 30+)
  - `fuse` - Fuse two Titans (requires Level 20+, same element)
  - `transfer` - Transfer Titan ownership
  - `update_config` - Admin configuration updates
  - `set_paused` - Pause/unpause program

#### Account Structures
- `GlobalConfig` (182 bytes) - Program configuration with packed repr
- `TitanData` (118 bytes) - Titan NFT on-chain data
- `PlayerAccount` (152 bytes) - Player profile and statistics

#### Testing
- TypeScript integration test suite (22 tests, 100% passing)
- Basic functionality tests (9 tests)
- Edge case tests (5 tests): Invalid inputs, self-fusion, max limits
- Authorization tests (3 tests): Unauthorized access rejection
- Error handling validation for all error codes

### Deployed
- **Devnet**: `3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7`

### Technical Details
- Framework: Pinocchio 0.8
- Program Size: ~23KB
- Zero-copy deserialization for performance
- CPI-based PDA account creation

---

## [0.1.0] - 2026-01-20

### Added

#### Documentation
- Complete game design document (`BREACH_DESIGN_DOCUMENT.md`)
- Technical specification (`TECHNICAL_SPECIFICATION.md`)
- Smart contract specification (`SMART_CONTRACT_SPECIFICATION.md`)
- Project README with overview and features

#### Website
- Landing page with hero section
- About section introducing the Linker concept
- Features section highlighting core gameplay
- Titans section with elemental types and threat classes
- Tokenomics section with $BREACH distribution
- Roadmap section with development phases
- Waitlist signup form
- Responsive navigation with mobile menu
- Footer with social links

#### Website Pages
- Whitepaper page with 9 detailed sections
- Documentation page with gameplay guides
- FAQ page with expandable answers
- Privacy Policy page
- Terms of Service page

#### SEO & Performance
- Dynamic sitemap generation
- Robots.txt configuration
- Open Graph images for social sharing
- PWA manifest for mobile installation
- Next.js Image optimization

#### Design
- Custom glassmorphism UI components
- Animated background effects (particles, orbs, scan lines)
- Responsive layout for all screen sizes
- Custom fonts (Orbitron, Rajdhani)
- Titan concept art integration

#### Infrastructure
- Vercel deployment configuration
- Git repository setup
- Project structure organization

### Technical Details
- Framework: Next.js 16 with Turbopack
- Styling: Tailwind CSS 4
- Animations: Framer Motion
- Icons: Lucide React
- Package Manager: pnpm

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| 0.6.0 | 2026-01-20 | PvP matchmaking, social features, guilds |
| 0.5.0 | 2026-01-20 | Backend API with core features |
| 0.4.0 | 2026-01-20 | Game economy & geographic system design |
| 0.3.1 | 2026-01-20 | $BREACH Token created on Devnet |
| 0.3.0 | 2026-01-20 | Game Logic Program deployed to Devnet |
| 0.2.0 | 2026-01-20 | Titan NFT Program deployed to Devnet |
| 0.1.0 | 2026-01-20 | Initial release with documentation and website |

---

## Upcoming Releases

### v0.7.0 (Planned)
- Trading marketplace
- Real-time chat system
- Enhanced WebSocket events

### v0.8.0 (Planned)
- Mobile app MVP (Flutter)
- AR capture prototype
- Push notifications

### v1.0.0 (Target)
- Mainnet launch
- Full game release
- Public token sale

---

## Links

- [GitHub Repository](https://github.com/vnxfsc/BREACH)
- [Live Website](https://breach-jade.vercel.app)
- [Documentation](https://breach-jade.vercel.app/docs)
- [Titan NFT Program (Devnet)](https://explorer.solana.com/address/3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7?cluster=devnet)
- [Game Logic Program (Devnet)](https://explorer.solana.com/address/DLk2GnDu9AYn7PeLprEDHDYH9UWKENX47UqqfeiQBaSX?cluster=devnet)

---

[Unreleased]: https://github.com/vnxfsc/BREACH/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/vnxfsc/BREACH/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/vnxfsc/BREACH/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/vnxfsc/BREACH/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/vnxfsc/BREACH/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/vnxfsc/BREACH/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/vnxfsc/BREACH/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/vnxfsc/BREACH/releases/tag/v0.1.0
