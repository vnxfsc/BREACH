# BREACH Geographic System Design

> Version 1.0 | January 2026

## Table of Contents

1. [Overview](#1-overview)
2. [POI System](#2-poi-system)
3. [Titan Spawn Algorithm](#3-titan-spawn-algorithm)
4. [Location Verification](#4-location-verification)
5. [Data Sources](#5-data-sources)
6. [Database Schema](#6-database-schema)
7. [API Design](#7-api-design)
8. [Global Deployment](#8-global-deployment)
9. [Performance Optimization](#9-performance-optimization)
10. [Privacy & Compliance](#10-privacy--compliance)

---

## 1. Overview

### 1.1 Core Philosophy

BREACH uses a **location-first** design where Titans spawn at fixed geographic points, not around players. This creates:

- **Exploration incentive** - Players must travel to locations
- **Fair competition** - Everyone sees the same Titans
- **Real-world value** - Locations gain significance
- **Social encounters** - Players meet at spawn points

### 1.2 System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Mobile App                              │
│                  (GPS + AR Camera)                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Backend Services                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  Map API    │  │  Spawn      │  │  Location   │         │
│  │  Service    │  │  Service    │  │  Validator  │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
        ┌──────────┐   ┌──────────┐   ┌──────────┐
        │PostgreSQL│   │  Redis   │   │ External │
        │ + PostGIS│   │  Cache   │   │   APIs   │
        └──────────┘   └──────────┘   └──────────┘
```

---

## 2. POI System

### 2.1 POI Categories

| Category | Examples | Spawn Weight | Typical Classes |
|----------|----------|--------------|-----------------|
| **Landmark** | Tokyo Tower, Eiffel Tower | 5.0 | III-V |
| **Tourist Attraction** | Museums, Temples | 4.0 | II-IV |
| **Park** | City parks, Gardens | 3.0 | I-III |
| **Public Square** | Plazas, Town centers | 2.5 | I-III |
| **Commercial** | Malls, Stations | 2.0 | I-II |
| **Educational** | Universities, Libraries | 1.5 | I-II |
| **Residential** | Neighborhoods | 0.5 | I only |

### 2.2 POI Data Structure

```rust
#[derive(Debug, Clone)]
pub struct PointOfInterest {
    pub id: Uuid,
    pub name: String,
    pub category: POICategory,
    pub location: GeoPoint,
    pub radius: f64,              // Spawn radius in meters
    pub spawn_weight: f64,        // Higher = more spawns
    pub terrain_type: TerrainType,
    pub population_density: u32,  // People per km²
    pub timezone: String,
    pub country_code: String,
    pub is_active: bool,
    pub metadata: POIMetadata,
}

#[derive(Debug, Clone)]
pub struct POIMetadata {
    pub osm_id: Option<String>,
    pub google_place_id: Option<String>,
    pub opening_hours: Option<String>,
    pub accessibility: bool,
    pub indoor: bool,
    pub elevation: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
pub enum TerrainType {
    Water,      // Ocean, Lake, River
    Mountain,   // Hills, Mountains
    Urban,      // City, Town
    Forest,     // Woods, Parks with trees
    Desert,     // Arid regions
    Coastal,    // Beach, Shore
    Arctic,     // Snow, Ice
}

#[derive(Debug, Clone, Copy)]
pub enum POICategory {
    Landmark,
    TouristAttraction,
    Park,
    PublicSquare,
    Commercial,
    Educational,
    Religious,
    Sports,
    Transportation,
    Residential,
}
```

### 2.3 POI Import Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│                    POI Import Pipeline                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Data Sources                                             │
│     ├── OpenStreetMap (Primary)                             │
│     ├── Google Places API (Verification)                    │
│     └── Manual Curation (Famous landmarks)                  │
│                                                              │
│  2. Processing                                               │
│     ├── Deduplication (same location different sources)     │
│     ├── Category Classification                             │
│     ├── Spawn Weight Calculation                            │
│     └── Terrain Type Assignment                             │
│                                                              │
│  3. Validation                                               │
│     ├── Coordinate Verification                             │
│     ├── Accessibility Check                                 │
│     └── Legal/Safety Review                                 │
│                                                              │
│  4. Storage                                                  │
│     └── PostgreSQL + PostGIS with spatial indexing          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.4 Excluded Locations

| Exclusion Type | Reason | Detection Method |
|----------------|--------|------------------|
| Private Property | Legal issues | Property boundaries |
| Military Bases | Security | OSM tags |
| Hospitals | Safety | Category filter |
| Schools (K-12) | Child safety | Category filter |
| Cemeteries | Respect | Category filter |
| Dangerous Areas | Safety | Manual curation |

---

## 3. Titan Spawn Algorithm

### 3.1 Global Spawn Scheduler

```rust
// Runs every hour on each regional server
pub async fn global_spawn_cycle(region: &Region) -> Result<Vec<TitanSpawn>> {
    let mut spawns = Vec::new();
    
    // Get all POIs in region
    let pois = get_active_pois(region).await?;
    
    for poi in pois {
        // Check if POI already has active Titan
        if has_active_titan(&poi).await? {
            continue;
        }
        
        // Calculate spawn probability
        let spawn_chance = calculate_spawn_probability(&poi);
        
        if random::<f64>() < spawn_chance {
            let titan = generate_titan_for_poi(&poi).await?;
            spawns.push(titan);
        }
    }
    
    // Batch insert spawns
    insert_titan_spawns(&spawns).await?;
    
    // Notify nearby players via WebSocket
    notify_spawn_events(&spawns).await?;
    
    Ok(spawns)
}
```

### 3.2 Spawn Probability Calculation

```rust
pub fn calculate_spawn_probability(poi: &PointOfInterest) -> f64 {
    let base_probability = 0.3; // 30% base chance per hour
    
    // Factor 1: POI Weight
    let weight_factor = poi.spawn_weight / 3.0; // Normalized
    
    // Factor 2: Time of Day (local)
    let local_hour = get_local_hour(&poi.timezone);
    let time_factor = match local_hour {
        6..=9 => 1.2,    // Morning rush
        12..=14 => 1.3,  // Lunch break
        17..=20 => 1.5,  // Evening peak
        22..=5 => 0.3,   // Night (reduced)
        _ => 1.0
    };
    
    // Factor 3: Day of Week
    let day_factor = if is_weekend() { 1.3 } else { 1.0 };
    
    // Factor 4: Weather (if available)
    let weather_factor = match get_weather(&poi.location) {
        Weather::Clear => 1.0,
        Weather::Cloudy => 1.1,
        Weather::Rain => 0.7,     // Fewer spawns in bad weather
        Weather::Storm => 1.5,    // Storm element bonus!
        Weather::Snow => 0.8,
    };
    
    // Factor 5: Recent Activity (demand-based)
    let activity = get_recent_player_activity(&poi).await;
    let activity_factor = (activity as f64 / 100.0).clamp(0.5, 2.0);
    
    base_probability * weight_factor * time_factor * day_factor * weather_factor * activity_factor
}
```

### 3.3 Titan Generation

```rust
pub async fn generate_titan_for_poi(poi: &PointOfInterest) -> Result<TitanSpawn> {
    // Determine element based on terrain
    let element = determine_element(poi);
    
    // Determine class based on POI category and rarity
    let threat_class = determine_threat_class(poi);
    
    // Calculate spawn position (within POI radius)
    let spawn_position = random_point_in_circle(poi.location, poi.radius);
    
    // Calculate expiration time based on class
    let duration = match threat_class {
        ThreatClass::I => Duration::hours(4),
        ThreatClass::II => Duration::hours(3),
        ThreatClass::III => Duration::hours(2),
        ThreatClass::IV => Duration::hours(1),
        ThreatClass::V => Duration::minutes(30),
    };
    
    Ok(TitanSpawn {
        id: Uuid::new_v4(),
        poi_id: poi.id,
        location: spawn_position,
        element,
        threat_class,
        spawned_at: Utc::now(),
        expires_at: Utc::now() + duration,
        captured_by: None,
        capture_count: 0, // For Class I-II, multiple captures allowed
    })
}
```

### 3.4 Element Assignment by Terrain

```rust
pub fn determine_element(poi: &PointOfInterest) -> Element {
    let weights = match poi.terrain_type {
        TerrainType::Water => vec![
            (Element::Abyssal, 70),
            (Element::Storm, 20),
            (Element::Parasitic, 10),
        ],
        TerrainType::Mountain => vec![
            (Element::Volcanic, 60),
            (Element::Storm, 25),
            (Element::Ossified, 15),
        ],
        TerrainType::Urban => vec![
            (Element::Storm, 40),
            (Element::Void, 35),
            (Element::Parasitic, 25),
        ],
        TerrainType::Forest => vec![
            (Element::Parasitic, 65),
            (Element::Ossified, 20),
            (Element::Abyssal, 15),
        ],
        TerrainType::Desert => vec![
            (Element::Volcanic, 50),
            (Element::Ossified, 35),
            (Element::Void, 15),
        ],
        TerrainType::Coastal => vec![
            (Element::Abyssal, 45),
            (Element::Storm, 35),
            (Element::Volcanic, 20),
        ],
        TerrainType::Arctic => vec![
            (Element::Ossified, 60),
            (Element::Void, 25),
            (Element::Storm, 15),
        ],
    };
    
    weighted_random_selection(&weights)
}
```

### 3.5 Threat Class Distribution

```rust
pub fn determine_threat_class(poi: &PointOfInterest) -> ThreatClass {
    // Base distribution
    let mut weights = vec![
        (ThreatClass::I, 60.0),
        (ThreatClass::II, 25.0),
        (ThreatClass::III, 10.0),
        (ThreatClass::IV, 4.0),
        (ThreatClass::V, 1.0),
    ];
    
    // Adjust based on POI category
    match poi.category {
        POICategory::Landmark => {
            // Landmarks have higher rare chance
            weights[2].1 *= 2.0;  // Class III: 20%
            weights[3].1 *= 3.0;  // Class IV: 12%
            weights[4].1 *= 5.0;  // Class V: 5%
        }
        POICategory::TouristAttraction => {
            weights[2].1 *= 1.5;
            weights[3].1 *= 2.0;
        }
        POICategory::Residential => {
            // Residential only spawns common
            weights[1].1 = 0.0;
            weights[2].1 = 0.0;
            weights[3].1 = 0.0;
            weights[4].1 = 0.0;
        }
        _ => {}
    }
    
    // Normalize and select
    let total: f64 = weights.iter().map(|(_, w)| w).sum();
    let normalized: Vec<_> = weights.iter()
        .map(|(c, w)| (*c, (w / total * 100.0) as u32))
        .collect();
    
    weighted_random_selection(&normalized)
}
```

### 3.6 Regional Density Balancing

```rust
/// Ensures fair Titan density across different population densities
pub fn calculate_regional_spawn_quota(region: &Region) -> u32 {
    let base_quota = 1000; // Base spawns per region per hour
    
    // Population adjustment (inverse relationship)
    // More people = fewer Titans per person to prevent city monopoly
    let pop_factor = match region.population_density {
        0..=100 => 2.0,       // Rural: double quota
        101..=500 => 1.5,     // Small town
        501..=2000 => 1.0,    // Medium city
        2001..=5000 => 0.8,   // Large city
        5001..=10000 => 0.6,  // Very large city
        _ => 0.4,             // Megacity
    };
    
    // Area adjustment
    let area_factor = (region.area_km2 / 100.0).sqrt().clamp(0.5, 3.0);
    
    // POI density bonus
    let poi_bonus = (region.poi_count as f64 / 50.0).clamp(0.8, 1.5);
    
    (base_quota as f64 * pop_factor * area_factor * poi_bonus) as u32
}
```

---

## 4. Location Verification

### 4.1 Anti-Spoofing System

```rust
pub struct LocationVerifier {
    max_speed_mps: f64,           // 42 m/s = 150 km/h
    teleport_threshold_km: f64,   // 50 km instant jump = suspicious
    accuracy_threshold_m: f64,    // Reject accuracy > 100m
    mock_detection_enabled: bool,
}

impl LocationVerifier {
    pub async fn verify_location(
        &self,
        player: &Player,
        claimed_location: GeoPoint,
        timestamp: DateTime<Utc>,
    ) -> Result<LocationVerification> {
        let mut flags = Vec::new();
        
        // 1. Check GPS accuracy
        if claimed_location.accuracy > self.accuracy_threshold_m {
            flags.push(Flag::LowAccuracy);
        }
        
        // 2. Check for mock location indicators
        if self.mock_detection_enabled && claimed_location.is_mock {
            return Err(Error::MockLocationDetected);
        }
        
        // 3. Calculate movement from last known position
        if let Some(last) = player.last_location {
            let distance = haversine_distance(&last.point, &claimed_location);
            let time_diff = timestamp - last.timestamp;
            let speed = distance / time_diff.as_secs_f64();
            
            // Speed check
            if speed > self.max_speed_mps {
                flags.push(Flag::SpeedViolation { speed, max: self.max_speed_mps });
            }
            
            // Teleport check
            if distance > self.teleport_threshold_km * 1000.0 
               && time_diff < Duration::minutes(5) {
                flags.push(Flag::PossibleTeleport { distance });
            }
        }
        
        // 4. Check against known VPN/datacenter IPs
        if is_suspicious_ip(&player.ip_address).await? {
            flags.push(Flag::SuspiciousIP);
        }
        
        // 5. Device sensor validation (accelerometer, gyroscope)
        if !validate_motion_sensors(&claimed_location.sensor_data) {
            flags.push(Flag::SensorMismatch);
        }
        
        // Determine result
        let status = if flags.is_empty() {
            VerificationStatus::Valid
        } else if flags.iter().any(|f| f.is_critical()) {
            VerificationStatus::Rejected
        } else {
            VerificationStatus::Suspicious
        };
        
        Ok(LocationVerification { status, flags })
    }
}
```

### 4.2 Capture Verification Flow

```
┌─────────────────────────────────────────────────────────────┐
│                  Capture Verification Flow                   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Player requests capture                                  │
│     └── Sends: player_location, titan_id, device_info       │
│                                                              │
│  2. Location Verification                                    │
│     ├── GPS accuracy check                                  │
│     ├── Speed/teleport check                                │
│     ├── Mock location detection                             │
│     └── IP reputation check                                 │
│                                                              │
│  3. Distance Validation                                      │
│     └── Player within 50m of Titan? (configurable)          │
│                                                              │
│  4. Titan State Check                                        │
│     ├── Titan still active?                                 │
│     ├── Not already captured?                               │
│     └── Player cooldown expired?                            │
│                                                              │
│  5. Generate Backend Signature                               │
│     └── Signs: player_wallet, titan_id, timestamp           │
│                                                              │
│  6. Return signature to client                               │
│     └── Valid for 5 minutes                                 │
│                                                              │
│  7. Client submits to blockchain                             │
│     └── Smart contract verifies signature                   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 4.3 Progressive Punishment System

| Offense Count | Action |
|---------------|--------|
| 1st | Warning + CAPTCHA required |
| 2nd | 24-hour capture ban |
| 3rd | 7-day capture ban |
| 4th | 30-day account suspension |
| 5th+ | Permanent ban |

```rust
pub async fn handle_cheating_detection(player: &Player, offense: Offense) -> Result<()> {
    let offense_count = increment_offense_count(player, &offense).await?;
    
    let punishment = match offense_count {
        1 => Punishment::Warning { require_captcha: true },
        2 => Punishment::CaptureBan { duration: Duration::hours(24) },
        3 => Punishment::CaptureBan { duration: Duration::days(7) },
        4 => Punishment::AccountSuspension { duration: Duration::days(30) },
        _ => Punishment::PermanentBan,
    };
    
    apply_punishment(player, punishment).await?;
    log_security_event(player, &offense, &punishment).await?;
    
    Ok(())
}
```

---

## 5. Data Sources

### 5.1 Primary: OpenStreetMap

```rust
pub struct OSMImporter {
    pub endpoint: String,
    pub bbox: BoundingBox,
}

impl OSMImporter {
    pub async fn import_pois(&self) -> Result<Vec<RawPOI>> {
        // Overpass API query for relevant POI types
        let query = r#"
            [out:json][timeout:300];
            (
                // Parks and nature
                way["leisure"="park"](bbox);
                way["landuse"="forest"](bbox);
                
                // Landmarks and tourism
                node["tourism"="attraction"](bbox);
                node["historic"](bbox);
                way["tourism"="museum"](bbox);
                
                // Public spaces
                way["place"="square"](bbox);
                node["amenity"="fountain"](bbox);
                
                // Commercial
                way["shop"="mall"](bbox);
                node["railway"="station"](bbox);
            );
            out center;
        "#;
        
        let response = self.query_overpass(&query).await?;
        parse_osm_response(response)
    }
}
```

### 5.2 Secondary: Google Places API

```rust
pub struct GooglePlacesClient {
    api_key: String,
    rate_limiter: RateLimiter,
}

impl GooglePlacesClient {
    /// Used for verification and enrichment, not primary import
    pub async fn verify_poi(&self, poi: &RawPOI) -> Result<PlaceVerification> {
        self.rate_limiter.acquire().await;
        
        let result = self.nearby_search(
            poi.location,
            50.0, // 50m radius
            &poi.name
        ).await?;
        
        Ok(PlaceVerification {
            exists: !result.is_empty(),
            google_place_id: result.first().map(|p| p.place_id.clone()),
            rating: result.first().and_then(|p| p.rating),
            opening_hours: result.first().and_then(|p| p.opening_hours.clone()),
        })
    }
}
```

### 5.3 Weather Data: OpenWeather API

```rust
pub struct WeatherService {
    api_key: String,
    cache: Cache<GeoPoint, WeatherData>,
}

impl WeatherService {
    pub async fn get_weather(&self, location: &GeoPoint) -> Result<WeatherData> {
        // Check cache first (15 min TTL)
        if let Some(cached) = self.cache.get(location) {
            return Ok(cached);
        }
        
        let response = reqwest::get(format!(
            "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}",
            location.lat, location.lng, self.api_key
        )).await?;
        
        let data: WeatherData = response.json().await?;
        self.cache.insert(location.clone(), data.clone());
        
        Ok(data)
    }
}

#[derive(Debug, Clone)]
pub enum Weather {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Snow,
    Fog,
}
```

### 5.4 Terrain Data: Natural Earth + OSM

```rust
pub async fn determine_terrain(location: &GeoPoint) -> TerrainType {
    // Check water bodies
    if is_in_water(location).await {
        return TerrainType::Water;
    }
    
    // Check elevation for mountains
    let elevation = get_elevation(location).await;
    if elevation > 1000.0 {
        return TerrainType::Mountain;
    }
    
    // Check land use from OSM
    let land_use = get_osm_landuse(location).await;
    match land_use.as_str() {
        "forest" | "wood" => TerrainType::Forest,
        "residential" | "commercial" | "industrial" => TerrainType::Urban,
        "desert" | "sand" => TerrainType::Desert,
        _ => TerrainType::Urban // Default
    }
}
```

---

## 6. Database Schema

### 6.1 PostgreSQL + PostGIS Schema

```sql
-- Enable PostGIS extension
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS pg_trgm; -- For text search

-- Regions table (for sharding and management)
CREATE TABLE regions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    country_code CHAR(2) NOT NULL,
    timezone VARCHAR(50) NOT NULL,
    bounds GEOGRAPHY(POLYGON, 4326) NOT NULL,
    population_density INT,
    spawn_quota INT DEFAULT 1000,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_regions_bounds ON regions USING GIST(bounds);

-- Points of Interest
CREATE TABLE pois (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    region_id UUID REFERENCES regions(id),
    name VARCHAR(500) NOT NULL,
    category VARCHAR(50) NOT NULL,
    location GEOGRAPHY(POINT, 4326) NOT NULL,
    radius FLOAT DEFAULT 50.0,
    spawn_weight FLOAT DEFAULT 1.0,
    terrain_type VARCHAR(20) NOT NULL,
    
    -- External IDs
    osm_id VARCHAR(50),
    google_place_id VARCHAR(100),
    
    -- Metadata
    opening_hours JSONB,
    is_indoor BOOLEAN DEFAULT false,
    accessibility BOOLEAN DEFAULT true,
    elevation FLOAT,
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    is_verified BOOLEAN DEFAULT false,
    last_verified_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_pois_location ON pois USING GIST(location);
CREATE INDEX idx_pois_region ON pois(region_id);
CREATE INDEX idx_pois_category ON pois(category);
CREATE INDEX idx_pois_name_trgm ON pois USING GIN(name gin_trgm_ops);

-- Active Titan Spawns
CREATE TABLE titan_spawns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poi_id UUID REFERENCES pois(id),
    location GEOGRAPHY(POINT, 4326) NOT NULL,
    element VARCHAR(20) NOT NULL,
    threat_class SMALLINT NOT NULL CHECK (threat_class BETWEEN 1 AND 5),
    
    -- Timing
    spawned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Capture state
    captured_by UUID, -- Player ID
    captured_at TIMESTAMPTZ,
    capture_count INT DEFAULT 0, -- For multi-capture Titans
    max_captures INT DEFAULT 1,
    
    -- Indexing
    geohash VARCHAR(12) NOT NULL -- For efficient spatial queries
);

CREATE INDEX idx_titan_spawns_location ON titan_spawns USING GIST(location);
CREATE INDEX idx_titan_spawns_expires ON titan_spawns(expires_at) WHERE captured_by IS NULL;
CREATE INDEX idx_titan_spawns_geohash ON titan_spawns(geohash);
CREATE INDEX idx_titan_spawns_active ON titan_spawns(expires_at) 
    WHERE captured_by IS NULL AND expires_at > NOW();

-- Player location history (for verification)
CREATE TABLE player_locations (
    id BIGSERIAL PRIMARY KEY,
    player_id UUID NOT NULL,
    location GEOGRAPHY(POINT, 4326) NOT NULL,
    accuracy FLOAT,
    speed FLOAT,
    heading FLOAT,
    altitude FLOAT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    device_id VARCHAR(100),
    
    -- Verification flags
    is_suspicious BOOLEAN DEFAULT false,
    flags JSONB
);

CREATE INDEX idx_player_locations_player ON player_locations(player_id, timestamp DESC);
CREATE INDEX idx_player_locations_time ON player_locations(timestamp);

-- Partitioning for player_locations (by month)
CREATE TABLE player_locations_2026_01 PARTITION OF player_locations
    FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');
```

### 6.2 Redis Cache Schema

```
# Active Titans by GeoHash (for fast spatial queries)
# Key: titans:geohash:{geohash_prefix}
# Value: Set of Titan IDs
SADD titans:geohash:u4pruyd {"titan_id_1", "titan_id_2"}
EXPIRE titans:geohash:u4pruyd 14400  # 4 hours

# Titan Details (avoid DB hits)
# Key: titan:{titan_id}
# Value: JSON blob
SET titan:abc123 '{"id":"abc123","location":{...},"element":"abyssal",...}'
EXPIRE titan:abc123 14400

# Player last location (for speed validation)
# Key: player:loc:{player_id}
# Value: JSON with location + timestamp
SET player:loc:xyz789 '{"lat":35.68,"lng":139.76,"ts":1706000000}'
EXPIRE player:loc:xyz789 3600

# POI spawn cooldown (prevent over-spawning)
# Key: poi:cooldown:{poi_id}
# Value: 1 (just existence check)
SET poi:cooldown:poi123 1
EXPIRE poi:cooldown:poi123 3600

# Player capture cooldown
# Key: player:cooldown:{player_id}
# Value: timestamp of last capture
SET player:cooldown:xyz789 1706000000
EXPIRE player:cooldown:xyz789 300  # 5 min cooldown
```

---

## 7. API Design

### 7.1 Map Endpoints

```yaml
# Get nearby Titans
GET /api/v1/map/titans
Parameters:
  lat: float (required)
  lng: float (required)
  radius: int (default: 500, max: 2000) # meters
Response:
  {
    "titans": [
      {
        "id": "uuid",
        "location": { "lat": 35.68, "lng": 139.76 },
        "element": "abyssal",
        "threat_class": 2,
        "distance": 123.5,
        "expires_at": "2026-01-20T12:00:00Z",
        "poi_name": "Shibuya Crossing"
      }
    ],
    "total": 5,
    "query_radius": 500
  }

# Get POIs in area
GET /api/v1/map/pois
Parameters:
  bounds: string (required) # "sw_lat,sw_lng,ne_lat,ne_lng"
Response:
  {
    "pois": [
      {
        "id": "uuid",
        "name": "Tokyo Tower",
        "category": "landmark",
        "location": { "lat": 35.65, "lng": 139.74 },
        "has_active_titan": true
      }
    ]
  }

# Report player location (for tracking)
POST /api/v1/map/location
Body:
  {
    "lat": 35.68,
    "lng": 139.76,
    "accuracy": 10.5,
    "speed": 1.2,
    "heading": 180,
    "timestamp": "2026-01-20T10:00:00Z"
  }
Response:
  {
    "status": "ok",
    "verification": "valid"
  }
```

### 7.2 Capture Endpoints

```yaml
# Request capture (get signature)
POST /api/v1/capture/request
Body:
  {
    "titan_id": "uuid",
    "player_location": { "lat": 35.68, "lng": 139.76, "accuracy": 5.0 }
  }
Response:
  {
    "authorized": true,
    "signature": "base64_encoded_signature",
    "expires_at": "2026-01-20T10:05:00Z",
    "titan": {
      "id": "uuid",
      "element": "abyssal",
      "threat_class": 2,
      "genes": "base64_gene_sequence"
    }
  }
Error Response:
  {
    "authorized": false,
    "error": "too_far",
    "distance": 75.5,
    "max_distance": 50.0
  }
```

### 7.3 WebSocket Events

```yaml
# Subscribe to area updates
{
  "type": "subscribe",
  "channel": "map",
  "geohash": "u4pruyd" # 7-char precision ~150m
}

# Titan spawn event
{
  "type": "titan_spawn",
  "data": {
    "titan_id": "uuid",
    "location": { "lat": 35.68, "lng": 139.76 },
    "element": "volcanic",
    "threat_class": 3,
    "expires_at": "2026-01-20T12:00:00Z"
  }
}

# Titan captured event
{
  "type": "titan_captured",
  "data": {
    "titan_id": "uuid",
    "captured_by": "player_name" # Optional, for announcements
  }
}

# Titan expired event
{
  "type": "titan_expired",
  "data": {
    "titan_id": "uuid"
  }
}
```

---

## 8. Global Deployment

### 8.1 Regional Server Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Global Architecture                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │ Americas    │    │   Europe    │    │ Asia-Pacific │     │
│  │ (Virginia)  │    │ (Frankfurt) │    │   (Tokyo)   │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│         │                 │                   │              │
│         └─────────────────┼───────────────────┘              │
│                           │                                  │
│                    ┌──────┴──────┐                          │
│                    │   Global    │                          │
│                    │   Router    │                          │
│                    │  (Anycast)  │                          │
│                    └─────────────┘                          │
│                                                              │
└─────────────────────────────────────────────────────────────┘

Player Request → Anycast DNS → Nearest Regional Server
```

### 8.2 Region Configuration

```rust
pub struct RegionConfig {
    pub id: String,
    pub name: String,
    pub server_location: String,
    pub covered_countries: Vec<String>,
    pub timezone_default: String,
    pub spawn_multiplier: f64,
    pub language: String,
}

pub const REGIONS: &[RegionConfig] = &[
    RegionConfig {
        id: "na-east",
        name: "North America East",
        server_location: "us-east-1",
        covered_countries: vec!["US", "CA"],
        timezone_default: "America/New_York",
        spawn_multiplier: 1.0,
        language: "en",
    },
    RegionConfig {
        id: "eu-west",
        name: "Europe West",
        server_location: "eu-west-1",
        covered_countries: vec!["GB", "FR", "DE", "ES", "IT", "NL", "BE"],
        timezone_default: "Europe/London",
        spawn_multiplier: 1.0,
        language: "en",
    },
    RegionConfig {
        id: "ap-northeast",
        name: "Asia Pacific Northeast",
        server_location: "ap-northeast-1",
        covered_countries: vec!["JP", "KR", "TW"],
        timezone_default: "Asia/Tokyo",
        spawn_multiplier: 1.2, // Higher density regions
        language: "ja",
    },
    // ... more regions
];
```

### 8.3 Cross-Region Considerations

| Aspect | Handling |
|--------|----------|
| Player Travel | Seamless handoff between regions |
| Leaderboards | Global + Regional rankings |
| Events | Time-zone aware scheduling |
| Data Sync | Eventual consistency for non-critical data |

---

## 9. Performance Optimization

### 9.1 Spatial Query Optimization

```sql
-- Use GeoHash for primary filtering, then precise distance
-- Much faster than pure PostGIS distance queries

-- Step 1: Get geohash neighbors (in application)
-- Step 2: Query by geohash prefix
SELECT * FROM titan_spawns
WHERE geohash LIKE 'u4pruyd%'
  AND expires_at > NOW()
  AND captured_by IS NULL
  AND ST_DWithin(
    location,
    ST_Point(139.76, 35.68)::geography,
    500  -- 500m radius
  );
```

### 9.2 Caching Strategy

```
┌─────────────────────────────────────────────────────────────┐
│                    Cache Layers                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  L1: In-Memory (Process)                                     │
│      - Hot POIs                                             │
│      - Player session data                                  │
│      - TTL: 1 minute                                        │
│                                                              │
│  L2: Redis Cluster                                           │
│      - Active Titans by geohash                             │
│      - Player locations                                     │
│      - TTL: 5-60 minutes                                    │
│                                                              │
│  L3: PostgreSQL                                              │
│      - Source of truth                                      │
│      - Spatial indexes                                      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 9.3 Expected Performance

| Operation | Target Latency | Method |
|-----------|---------------|--------|
| Get nearby Titans | < 50ms | Redis + Geohash |
| Verify location | < 20ms | In-memory rules |
| Generate capture signature | < 100ms | Direct DB |
| WebSocket broadcast | < 10ms | Pub/sub |

---

## 10. Privacy & Compliance

### 10.1 GDPR Compliance

| Data Type | Retention | User Rights |
|-----------|-----------|-------------|
| Location History | 30 days | Export, Delete |
| Device Info | 30 days | Export, Delete |
| Play History | Indefinite | Export |
| Account Data | Until deletion | Full control |

### 10.2 Data Minimization

```rust
/// Only collect what's necessary
pub struct MinimalLocationData {
    pub lat: f64,           // Required for gameplay
    pub lng: f64,           // Required for gameplay
    pub accuracy: f64,      // Required for verification
    // NOT collected:
    // - Exact street address
    // - Place names
    // - Movement patterns beyond 30 days
}
```

### 10.3 Regional Restrictions

| Region | Restriction | Implementation |
|--------|-------------|----------------|
| China | Requires ICP license | Separate deployment or block |
| Germany | Strict GPS consent | Explicit opt-in |
| California | CCPA compliance | Data request portal |
| Children | COPPA (under 13) | Age gate, no precise location |

---

## Appendix A: GeoHash Reference

```
GeoHash precision levels:

Length | Cell Size      | Use Case
-------|----------------|------------------
1      | ~5,000 km      | Continental
2      | ~1,250 km      | Country
3      | ~156 km        | State/Province  
4      | ~39 km         | City
5      | ~4.9 km        | District
6      | ~1.2 km        | Neighborhood
7      | ~153 m         | Street block     ← Primary use
8      | ~38 m          | Building
9      | ~4.8 m         | Room
```

---

## Appendix B: Element-Terrain Matrix

| Terrain \ Element | Abyssal | Volcanic | Storm | Void | Parasitic | Ossified |
|-------------------|---------|----------|-------|------|-----------|----------|
| Water | **70%** | 5% | 15% | 5% | 5% | 0% |
| Mountain | 5% | **60%** | 20% | 5% | 5% | 5% |
| Urban | 5% | 5% | **35%** | **35%** | 15% | 5% |
| Forest | 10% | 5% | 5% | 5% | **65%** | 10% |
| Desert | 0% | **50%** | 5% | 10% | 5% | **30%** |
| Coastal | **40%** | 15% | **30%** | 5% | 5% | 5% |
| Arctic | 10% | 5% | 15% | 15% | 5% | **50%** |

---

*Last Updated: January 2026*
*Version: 1.0*
