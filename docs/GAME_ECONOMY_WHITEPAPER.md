# BREACH Game Economy Whitepaper

> Version 1.0 | January 2026

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Stakeholder Analysis](#2-stakeholder-analysis)
3. [Token Economics](#3-token-economics)
4. [Titan NFT Economy](#4-titan-nft-economy)
5. [Geographic System](#5-geographic-system)
6. [Player Progression](#6-player-progression)
7. [Monetization Model](#7-monetization-model)
8. [Anti-Cheat System](#8-anti-cheat-system)
9. [Social Systems](#9-social-systems)
10. [Emergency Protocols](#10-emergency-protocols)
11. [Key Metrics & Monitoring](#11-key-metrics--monitoring)

---

## 1. Executive Summary

BREACH is a location-based AR game where players hunt and capture massive creatures called Titans. This whitepaper outlines the economic systems designed to ensure:

- **Sustainable gameplay** for all player types
- **Token value preservation** for investors
- **Fair competition** regardless of spending
- **Long-term project viability**

### Core Principles

```
┌─────────────────────────────────────────────────────────────┐
│                    Economic Balance                          │
│                                                              │
│   Supply Control + Consumption Mechanisms = Value Stability  │
│                                                              │
│   ┌─────────────┐      ┌─────────────┐      ┌─────────────┐ │
│   │  Generation │  ≤   │ Consumption │  +   │ New Players │ │
│   │   (Mint)    │      │  (Burn/Use) │      │  (Demand)   │ │
│   └─────────────┘      └─────────────┘      └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Stakeholder Analysis

### 2.1 Player Segments

#### Free-to-Play Players (80% of users)

| Concern | Solution |
|---------|----------|
| "Can I play without paying?" | Daily guaranteed Titan spawns |
| "Will paying players crush me?" | Pay for convenience, not power |
| "Can I earn through time investment?" | More exploration = more rewards |

**Design Principles:**
- Guaranteed minimum 1 Titan capture per day
- Rare Titans obtained through "first come, first served" at locations
- Time investment directly correlates to earnings

#### Paying Players (15% of users)

| Concern | Solution |
|---------|----------|
| "Is spending worth it?" | Clear value proposition |
| "Will I be exploited?" | Transparent pricing, no gambling |

**What they CAN buy:**
- ✅ Convenience items (teleport, bag space)
- ✅ Cosmetic skins
- ✅ Season pass

**What they CANNOT buy:**
- ❌ High-tier Titans directly
- ❌ Combat stat boosts
- ❌ Ranking points

#### Hardcore Players (5% of users)

| Concern | Solution |
|---------|----------|
| "Is there depth?" | Complex breeding/fusion systems |
| "Is endgame challenging?" | Server-limited legendary Titans |

**Motivation Systems:**
- Global leaderboards (captures, battles, collection)
- Guild competitions
- World Boss events
- Exclusive titles and achievements

### 2.2 Investors / Token Holders

| Concern | Solution |
|---------|----------|
| "Will token inflate to zero?" | Deflationary consumption mechanics |
| "What's the utility?" | Required for all core actions |
| "Will team dump tokens?" | 2-year vesting, transparent treasury |

### 2.3 Project Team

| Concern | Solution |
|---------|----------|
| "How do we sustain operations?" | Multiple revenue streams |
| "How do we pay developers?" | Service-based income, not token sales |

---

## 3. Token Economics

### 3.1 $BREACH Token Overview

| Property | Value |
|----------|-------|
| Token Name | BREACH |
| Total Supply | 1,000,000,000 |
| Blockchain | Solana |
| Decimals | 9 |
| Mint Address | `CSH2Vz4MbgTLzB9SYJ7gBwNsyu7nKpbvEJzKQLgmmjt4` |

### 3.2 Token Allocation

| Category | Percentage | Amount | Vesting |
|----------|------------|--------|---------|
| Play-to-Earn | 35% | 350M | 4-year linear release |
| Ecosystem | 25% | 250M | DAO governance |
| Team | 15% | 150M | 2-year cliff + 2-year release |
| Treasury | 10% | 100M | Governance controlled |
| Liquidity | 10% | 100M | DEX pools at TGE |
| Advisors | 5% | 50M | 1-year vesting |

### 3.3 Token Flow Model

```
┌─────────────────────────────────────────────────────────────┐
│                    Daily Token Flow                          │
│                    (100K DAU Example)                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  【EMISSION】 5,000,000 $BREACH/day                         │
│                                                              │
│    Capture Rewards    2,000,000 (40%)                       │
│    Battle Rewards     1,500,000 (30%)                       │
│    Quest Rewards      1,000,000 (20%)                       │
│    Staking Yields       500,000 (10%)                       │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  【CONSUMPTION】 5,500,000 $BREACH/day                       │
│                                                              │
│    Capture Fees       1,650,000 (30%)                       │
│    Level-up Fees      1,375,000 (25%)                       │
│    Fusion Fees        1,100,000 (20%)                       │
│    Market Fees (BURN)   825,000 (15%)                       │
│    Item Purchases       550,000 (10%)                       │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  【NET FLOW】 -500,000 $BREACH/day = Mild Deflation ✅      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 3.4 Dynamic Emission Control

```rust
fn calculate_daily_emission() -> u64 {
    let base_emission = 5_000_000;
    
    // Factor 1: Active players
    let dau = get_daily_active_users();
    let player_modifier = match dau {
        0..=10_000 => 0.5,
        10_001..=50_000 => 1.0,
        50_001..=200_000 => 1.5,
        _ => 2.0
    };
    
    // Factor 2: Token price (prevent inflation)
    let price_usd = get_token_price();
    let price_modifier = match price_usd {
        0.0..=0.001 => 0.5,   // Price too low, reduce supply
        0.001..=0.01 => 1.0,  // Healthy range
        0.01..=0.1 => 1.2,    // Price high, slight increase
        _ => 1.5              // Price very high, increase supply
    };
    
    // Factor 3: Consumption rate
    let daily_burn = get_daily_burn();
    let burn_ratio = daily_burn as f64 / base_emission as f64;
    let consumption_modifier = burn_ratio.min(1.5);
    
    (base_emission as f64 * player_modifier * price_modifier * consumption_modifier) as u64
}
```

### 3.5 Token Utility

| Use Case | Cost | Effect |
|----------|------|--------|
| Capture Fee | 5-500 $BREACH | Required to capture Titans |
| Level Up | 50-5,000 $BREACH | Increase Titan level |
| Evolution | 1,000-10,000 $BREACH | Evolve to next form |
| Fusion | 500-5,000 $BREACH | Combine two Titans |
| Market Fee | 5% of sale | Partially burned |
| Premium Features | Various | Bag space, teleport, etc. |

---

## 4. Titan NFT Economy

### 4.1 Rarity Distribution

```
┌─────────────────────────────────────────────────────────────┐
│                   Titan Rarity Pyramid                       │
│                                                              │
│                          ▲                                   │
│                         /█\   Class V: 0.1%                 │
│                        /███\  Legendary - Event only        │
│                       /█████\                                │
│                      /███████\ Class IV: 0.9%               │
│                     /█████████\ Epic - Famous landmarks     │
│                    /███████████\                             │
│                   /█████████████\ Class III: 9%             │
│                  /███████████████\ Rare - Tourist spots     │
│                 /█████████████████\                          │
│                /███████████████████\ Class II: 30%          │
│               /█████████████████████\ Uncommon - Parks      │
│              /███████████████████████\                       │
│             /█████████████████████████\ Class I: 60%        │
│            /███████████████████████████\ Common - Anywhere  │
│           ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔                    │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Daily Spawn & Consumption Balance

```
Daily Titan Flow (100K DAU)

GENERATION: 50,000 Titans/day
├── Class I:   30,000 (60%)
├── Class II:  12,500 (25%)
├── Class III:  4,500 (9%)
├── Class IV:   2,500 (5%)
└── Class V:      500 (1%) - Special conditions only

CONSUMPTION: 35,000 Titans/day
├── Fusion:       20,000 (2→1, net -10,000)
├── Level Material: 10,000
└── Quest Sacrifice: 5,000

NET GROWTH: +15,000 Titans/day
├── Held by players: 12,000
└── Market circulation: 3,000

Result: Controlled growth ensures new players can obtain Titans
        while fusion mechanism limits total supply growth
```

### 4.3 Phased Generation Strategy

| Phase | Timeline | Daily Generation | Strategy |
|-------|----------|------------------|----------|
| **Launch** | 0-3 months | High (100K/day) | Attract users, low barrier |
| **Growth** | 3-12 months | Medium (50K/day) | Balance supply/demand |
| **Mature** | 1 year+ | Low (20K/day) | Maintain scarcity, rely on fusion consumption |

### 4.4 NFT Value Protection

| Mechanism | Implementation | Effect |
|-----------|----------------|--------|
| Supply Cap | Daily generation limits | Prevent inflation |
| Burn Mechanism | Fusion 2→1 | Total supply reduction |
| Rarity Lock | Class V server-limited | Never increase |
| Utility Value | Battle/earning capability | Not just collectibles |

---

## 5. Geographic System

### 5.1 Spawn Point Philosophy

**NOT player-centric spawning. Map-based fixed spawn points.**

| Aspect | Pokemon GO Style | BREACH Design |
|--------|-----------------|---------------|
| Spawn Location | Random around player | Fixed POI locations |
| Visibility | Only when nearby | Marked on map |
| Fairness | ❌ Stay home and play | ✅ Must travel to locations |
| Competition | ❌ Each player sees different | ✅ Same Titan for everyone |

### 5.2 Geographic Fairness Algorithm

```rust
fn calculate_region_spawn_density(region: &Region) -> f64 {
    let base_density = 10.0; // Base Titans per km²
    
    // Inverse population adjustment
    // Dense areas = fewer Titans per person (prevent city monopoly)
    let population_factor = match region.population_per_km2 {
        0..=100 => 2.0,      // Rural: 2x density
        101..=1000 => 1.5,   // Small town
        1001..=5000 => 1.0,  // Medium city
        5001..=10000 => 0.8, // Large city
        _ => 0.6             // Megacity
    };
    
    // POI bonus
    let poi_bonus = (region.poi_count as f64 / 100.0).min(1.5);
    
    base_density * population_factor * poi_bonus
}
```

**Result:**
- Tokyo downtown: Many people, few Titans per person → Competition
- Hokkaido countryside: Few people, many Titans per person → Travel incentive

### 5.3 Terrain-Based Element Assignment

| Terrain Type | Primary Element | Secondary |
|--------------|-----------------|-----------|
| Water/Ocean | Abyssal (70%) | Storm (30%) |
| Mountains | Volcanic (70%) | Storm (30%) |
| Urban | Storm (50%) | Void (50%) |
| Forest | Parasitic (70%) | Ossified (30%) |
| Desert | Volcanic (60%) | Ossified (40%) |

### 5.4 Titan Lifecycle

```
SPAWN ──────────────────────────────────────────────► EXPIRE
 │                                                      │
 │  ┌──────────────────────────────────────────────┐   │
 │  │         Titan Active Period (4 hours)        │   │
 │  │                                               │   │
 │  │  Player A sees it ────┐                      │   │
 │  │  Player B sees it ────┼── Same Titan         │   │
 │  │  Player C sees it ────┘                      │   │
 │  │                                               │   │
 │  │  Player B arrives first → Captures → Gone    │   │
 │  │  Player A/C arrive later → Already captured  │   │
 │  └──────────────────────────────────────────────┘   │
 │                                                      │
 ▼                                                      ▼
```

### 5.5 Spawn Timing by Class

| Class | Spawn Locations | Active Duration | Frequency |
|-------|-----------------|-----------------|-----------|
| I (Common) | Any POI | 4 hours | Hourly |
| II (Uncommon) | Parks, Squares | 3 hours | Every 2 hours |
| III (Rare) | Landmarks, Attractions | 2 hours | Every 4 hours |
| IV (Epic) | Famous Landmarks | 1 hour | 1-2 per day globally |
| V (Legendary) | Special Events | 30 minutes | Event-only |

---

## 6. Player Progression

### 6.1 New Player Journey (Days 1-7)

```
┌─────────────────────────────────────────────────────────────┐
│                    Newbie 7-Day Journey                      │
├─────────────────────────────────────────────────────────────┤
│ Day 1: First Capture                                         │
│   - Tutorial zone: Guaranteed Class I Titan spawn           │
│   - Simplified Neural Link (100% success rate)              │
│   - Reward: 100 $BREACH + First Titan                       │
│                                                              │
│ Day 2: Exploration                                           │
│   - Guide to nearby POI for second capture                  │
│   - Unlock Titan details page, attribute system             │
│   - Reward: 50 $BREACH                                      │
│                                                              │
│ Day 3: Combat Introduction                                   │
│   - Unlock PvE battles (vs wild Titans)                     │
│   - Gain battle experience                                  │
│   - Reward: First win bonus 100 $BREACH                     │
│                                                              │
│ Day 4-6: Growth Loop                                         │
│   - Daily quest system unlocks                              │
│   - Level up first Titan                                    │
│   - Experience element counters                             │
│                                                              │
│ Day 7: Social Introduction                                   │
│   - Unlock friend system                                    │
│   - Invite friend rewards                                   │
│   - 7-day login reward: Guaranteed Class II Titan           │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 Newbie Protection Mechanisms

| Mechanism | Duration | Effect |
|-----------|----------|--------|
| Protected Spawn Zone | First 3 days | Newbie-exclusive spawns (no veteran competition) |
| Success Rate Boost | First 7 days | Neural Link difficulty -50% |
| Daily Guarantee | Permanent | At least 1 Class I capture per day |
| PvP Matchmaking Shield | First 14 days | Only matched with similar-tenure players |

### 6.3 Long-term Player Goals

```
┌─────────────────────────────────────────────────────────────┐
│                      Goal Pyramid                            │
│                                                              │
│                          ▲                                   │
│                         /█\  Ultimate Goal                   │
│                        /███\ Collect all Class V (<100 ppl) │
│                       /█████\                                │
│                      /███████\ Long-term Goal               │
│                     /█████████\ Season rankings, guild      │
│                    /███████████\                             │
│                   /█████████████\ Mid-term Goal             │
│                  /███████████████\ Complete Pokedex, evolve │
│                 /█████████████████\                          │
│                /███████████████████\ Weekly Goal            │
│               /█████████████████████\ Weekly quests, events │
│              /███████████████████████\                       │
│             /█████████████████████████\ Daily Goal          │
│            /███████████████████████████\ Daily capture/battle│
│           ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔                    │
└─────────────────────────────────────────────────────────────┘
```

### 6.4 Season System (Every 3 Months)

| Content | Description |
|---------|-------------|
| Season Pass | Free + Premium tracks with rewards |
| Season-Exclusive Titans | Limited elements/skins, retired after season |
| Leaderboard Reset | Fresh competition each season |
| Season Recap | Statistics dashboard, achievement badges |

---

## 7. Monetization Model

### 7.1 Revenue Streams

| Source | Percentage | Description |
|--------|------------|-------------|
| Marketplace Fees | 40% | 5% on NFT trades |
| Item Sales | 30% | Convenience items, cosmetics |
| Staking Share | 15% | Portion of staking pool yield |
| Sponsorships | 10% | Brand collaborations, location partnerships |
| Subscriptions | 5% | Optional premium membership |

### 7.2 Purchasable Items

| Item | Price | Effect | Balance Impact |
|------|-------|--------|----------------|
| Teleport Ticket | $1 | Teleport to any Titan | ❌ Time saver only |
| Capture Ball Upgrade | $0.5 | +5% capture rate | ⚠️ Mild, capped |
| Bag Expansion | $2 | +10 Titan capacity | ❌ Convenience |
| Cosmetic Skins | $5-20 | Visual changes | ❌ Pure cosmetic |
| Season Pass | $10/season | Extra reward track | ❌ No power advantage |
| Monthly Card | $5/month | 50 $BREACH daily | ⚠️ Controlled range |

### 7.3 What We NEVER Sell

- ❌ High-tier Titans directly
- ❌ Combat stat boosts
- ❌ Ranking points or leaderboard advantages
- ❌ Loot boxes with random Titans

---

## 8. Anti-Cheat System

### 8.1 Cheat Types & Detection

| Cheat Type | Detection Method | Penalty |
|------------|------------------|---------|
| GPS Spoofing | Location jump detection, speed anomalies | Warning → 24h ban → Permanent |
| Botting | Behavior pattern analysis, CAPTCHA | Earnings reset + temp ban |
| Multi-accounting | Device fingerprint, IP correlation | All linked accounts banned |
| Market Manipulation | Abnormal trade monitoring | Asset freeze + investigation |

### 8.2 Location Validation Algorithm

```rust
fn validate_location_update(player: &Player, new_location: Location) -> Result<()> {
    let time_diff = now() - player.last_location_time;
    let distance = haversine(player.last_location, new_location);
    
    // Max speed: 150 km/h (allow for trains/planes with verification)
    let max_distance = time_diff.as_secs() as f64 * 41.67; // 150km/h in m/s
    
    if distance > max_distance * 1.2 { // 20% tolerance
        flag_suspicious_movement(player);
        
        if player.suspicious_count >= 3 {
            require_captcha(player)?;
        }
    }
    
    // Additional checks
    if is_known_vpn_location(new_location) {
        flag_vpn_usage(player);
    }
    
    Ok(())
}
```

### 8.3 Capture Verification Flow

```
Player Location (GPS) → Backend Verification → Signature Generation → On-chain Mint

┌─────────────────────────────────────────────────────────────┐
│                  Capture Verification                        │
│                                                              │
│  1. Player claims location (lat, lng)                       │
│  2. Backend checks:                                         │
│     - Distance to Titan < 50m                               │
│     - Movement history plausible                            │
│     - No suspicious patterns                                │
│  3. If valid: Generate backend signature                    │
│  4. Player submits signature to blockchain                  │
│  5. Smart contract verifies signature before minting        │
└─────────────────────────────────────────────────────────────┘
```

---

## 9. Social Systems

### 9.1 Friend System

| Feature | Description | Reward |
|---------|-------------|--------|
| Invite Friends | Registration with invite code | Both get 100 $BREACH |
| Team Capture | 2-4 players, same area | +50% EXP, +10% capture rate |
| Friendly Battle | No ranking loss sparring | Both gain experience |
| Daily Gifts | Send 3 items per day | Build friendship level |

### 9.2 Guild System

```
Guild Levels & Benefits:

Lv1 (10 members): Guild chat
Lv2 (25 members): Guild storage (shared items)
Lv3 (50 members): Guild quests (bonus rewards)
Lv4 (100 members): Guild wars (territory control)
Lv5 (200 members): Guild-exclusive Titan spawn points
```

### 9.3 World Boss Events

```
┌─────────────────────────────────────────────────────────────┐
│                    World Boss Event                          │
│                                                              │
│  📍 Location: Famous landmark (Tokyo Tower, Statue of Liberty)│
│  ⏰ Time: Every Saturday 20:00 local time                   │
│  👥 Participation: On-site players + Remote support         │
│                                                              │
│  Boss HP: 10,000,000                                         │
│                                                              │
│  ┌─────────────────────────────────────────┐                │
│  │  Player A: Damage 50,000 → Reward 500   │                │
│  │  Player B: Damage 120,000 → Reward 1,200│                │
│  │  Player C: Final blow → Bonus + Title   │                │
│  └─────────────────────────────────────────┘                │
│                                                              │
│  Rewards distributed by damage contribution:                 │
│  - $BREACH pool                                             │
│  - Rare material drops                                      │
│  - Limited titles/cosmetics                                 │
└─────────────────────────────────────────────────────────────┘
```

---

## 10. Emergency Protocols

### 10.1 Crisis Response Matrix

| Situation | Trigger | Response |
|-----------|---------|----------|
| Token Price Crash | -50% in 24h | Pause withdrawals, increase burn, buyback |
| NFT Floor Collapse | -90% floor price | Pause new generation, limited buyback |
| Mass Cheating | Abnormal data spike | Suspend affected features, rollback |
| Server Outage | Downtime > 1h | Compensate all players, extend events |

### 10.2 Economic Circuit Breakers

```rust
fn check_economic_health() -> HealthStatus {
    let metrics = get_daily_metrics();
    
    // Circuit breaker conditions
    if metrics.token_price_change_24h < -0.3 {
        return HealthStatus::Critical("Token price -30%+");
    }
    
    if metrics.daily_mints > metrics.daily_burns * 2.0 {
        return HealthStatus::Warning("Inflation risk");
    }
    
    if metrics.active_users < metrics.yesterday_active_users * 0.5 {
        return HealthStatus::Warning("Player exodus");
    }
    
    HealthStatus::Healthy
}
```

---

## 11. Key Metrics & Monitoring

### 11.1 Daily Health Dashboard

```rust
struct GameHealthMetrics {
    // Supply/Demand Balance
    daily_titan_mints: u32,       // Target: Proportional to DAU
    daily_titan_burns: u32,       // Target: ≥50% of mints
    
    // Economic Indicators
    token_price_change: f64,      // Target: -5% to +10%
    trading_volume: u64,          // Target: Stable growth
    floor_price_usd: f64,         // Target: Stable or growing
    
    // Player Behavior
    new_players: u32,             // Target: Continuous growth
    retention_d1: f64,            // Target: >40%
    retention_d7: f64,            // Target: >30%
    retention_d30: f64,           // Target: >15%
    avg_titans_per_player: f64,   // Target: 3-10
    
    // Geographic Distribution
    active_regions: u32,          // Target: Global spread
    gini_coefficient: f64,        // Target: <0.5 (fairness)
}
```

### 11.2 Success Criteria

| Dimension | Check | Target |
|-----------|-------|--------|
| Newbies | Can experience core fun in 7 days? | ✅ |
| F2P Players | Can play sustainably without paying? | ✅ |
| Paying Players | Spending doesn't break balance? | ✅ |
| Investors | Token has value support? | ✅ |
| Project Team | Sustainable income? | ✅ |
| Veterans | Long-term goals exist? | ✅ |
| Social | Motivation for interaction? | ✅ |
| Fairness | Global players have equal opportunity? | ✅ |
| Security | Anti-cheat mechanisms complete? | ✅ |
| Emergency | Response protocols ready? | ✅ |

---

## Conclusion

The BREACH economy is designed around sustainable value creation for all stakeholders. By balancing token emission with consumption, controlling NFT supply through fusion mechanics, and ensuring geographic fairness, we create an ecosystem where:

1. **Players** enjoy the game regardless of spending level
2. **Investors** see value preserved through deflationary mechanics
3. **The project** generates sustainable revenue through services
4. **The community** grows through social features and fair competition

This whitepaper will be updated as the game evolves and real-world data informs our economic models.

---

*Last Updated: January 2026*
*Version: 1.0*
