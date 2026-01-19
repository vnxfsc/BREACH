# Changelog

All notable changes to BREACH will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Planned
- Game Logic Program development
- Mobile app development (Flutter)
- Backend API implementation
- AR capture system integration

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
- TypeScript integration test suite (14 tests, 100% passing)
- Test coverage for all instructions
- Error handling validation

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
| 0.2.0 | 2026-01-20 | Titan NFT Program deployed to Devnet |
| 0.1.0 | 2026-01-20 | Initial release with documentation and website |

---

## Upcoming Releases

### v0.3.0 (Planned)
- Game Logic Program development
- Battle system implementation
- Experience/reward mechanics

### v0.4.0 (Planned)
- Backend API development
- Database schema implementation
- Authentication system

### v0.5.0 (Planned)
- Mobile app MVP
- AR capture prototype
- Wallet integration

### v1.0.0 (Target)
- Mainnet launch
- Full game release
- Public token sale

---

## Links

- [GitHub Repository](https://github.com/vnxfsc/BREACH)
- [Live Website](https://breach-jade.vercel.app)
- [Documentation](https://breach-jade.vercel.app/docs)
- [Devnet Program](https://explorer.solana.com/address/3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7?cluster=devnet)

---

[Unreleased]: https://github.com/vnxfsc/BREACH/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/vnxfsc/BREACH/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/vnxfsc/BREACH/releases/tag/v0.1.0
