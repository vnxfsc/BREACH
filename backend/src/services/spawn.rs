//! Titan spawn service

use chrono::{Datelike, Duration, Utc};
use rand::Rng;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::Database;
use crate::error::ApiResult;
use crate::models::{Element, POI, TerrainType, TitanSpawn};

/// Spawn service for generating Titans
#[derive(Clone)]
pub struct SpawnService {
    config: AppConfig,
    db: Database,
}

impl SpawnService {
    pub fn new(config: AppConfig, db: Database) -> Self {
        Self { config, db }
    }

    /// Run spawn cycle for a region
    pub async fn run_spawn_cycle(&self, region_id: Option<Uuid>) -> ApiResult<Vec<TitanSpawn>> {
        let mut spawns = Vec::new();

        // Get all active POIs
        let pois = self.get_eligible_pois(region_id).await?;

        for poi in pois {
            // Check if POI already has active Titan
            if self.poi_has_active_titan(poi.id).await? {
                continue;
            }

            // Calculate spawn probability
            let spawn_chance = self.calculate_spawn_probability(&poi);

            // Generate random outside of async context
            let should_spawn = {
                let mut rng = rand::thread_rng();
                rng.gen::<f64>() < spawn_chance
            };

            if should_spawn {
                let titan = self.generate_titan_for_poi(&poi).await?;
                spawns.push(titan);
            }
        }

        tracing::info!("Spawn cycle complete: {} new Titans", spawns.len());

        Ok(spawns)
    }

    /// Get POIs eligible for spawning
    async fn get_eligible_pois(&self, region_id: Option<Uuid>) -> ApiResult<Vec<POI>> {
        let pois = if let Some(rid) = region_id {
            sqlx::query_as::<_, POI>(
                r#"
                SELECT * FROM pois WHERE is_active = true AND region_id = $1
                "#,
            )
            .bind(rid)
            .fetch_all(&self.db.pg)
            .await?
        } else {
            sqlx::query_as::<_, POI>(
                r#"
                SELECT * FROM pois WHERE is_active = true
                "#,
            )
            .fetch_all(&self.db.pg)
            .await?
        };

        Ok(pois)
    }

    /// Check if POI has an active Titan
    async fn poi_has_active_titan(&self, poi_id: Uuid) -> ApiResult<bool> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM titan_spawns 
            WHERE poi_id = $1 
              AND expires_at > NOW()
              AND (captured_by IS NULL OR capture_count < max_captures)
            "#,
        )
        .bind(poi_id)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(count > 0)
    }

    /// Calculate spawn probability for a POI
    fn calculate_spawn_probability(&self, poi: &POI) -> f64 {
        let base_probability = 0.3; // 30% base chance per hour

        // Factor 1: POI Weight
        let weight_factor = poi.spawn_weight / 3.0;

        // Factor 2: Time of day (simplified)
        let hour = Utc::now().hour();
        let time_factor = match hour {
            6..=9 => 1.2,
            12..=14 => 1.3,
            17..=20 => 1.5,
            22..=23 | 0..=5 => 0.3,
            _ => 1.0,
        };

        // Factor 3: Day of week
        let day_factor = if Utc::now().weekday().num_days_from_monday() >= 5 {
            1.3
        } else {
            1.0
        };

        base_probability * weight_factor * time_factor * day_factor
    }

    /// Generate a Titan for a POI
    async fn generate_titan_for_poi(&self, poi: &POI) -> ApiResult<TitanSpawn> {
        // Generate all random values BEFORE any await
        let (element, threat_class, spawn_lat, spawn_lng, geohash, species_id, genes, max_captures, duration) = {
            let mut rng = rand::thread_rng();

            // Determine element based on terrain
            let element = self.determine_element_sync(poi.terrain_type, &mut rng);

            // Determine threat class
            let threat_class = self.determine_threat_class_sync(poi, &mut rng);

            // Generate random position within POI radius
            let angle = rng.gen::<f64>() * 2.0 * std::f64::consts::PI;
            let distance = rng.gen::<f64>() * poi.radius;
            let offset_lat = (distance * angle.cos()) / 111_320.0;
            let offset_lng =
                (distance * angle.sin()) / (111_320.0 * poi.location_lat.to_radians().cos());

            let spawn_lat = poi.location_lat + offset_lat;
            let spawn_lng = poi.location_lng + offset_lng;

            // Generate geohash
            let geohash = geohash::encode(
                geohash::Coord {
                    x: spawn_lng,
                    y: spawn_lat,
                },
                7,
            )
            .unwrap_or_default();

            // Calculate duration based on threat class
            let duration = match threat_class {
                1 => Duration::hours(4),
                2 => Duration::hours(3),
                3 => Duration::hours(2),
                4 => Duration::hours(1),
                5 => Duration::minutes(30),
                _ => Duration::hours(2),
            };

            // Generate species ID and genes
            let species_id = self.generate_species_id_sync(element, threat_class, &mut rng);
            let genes = self.generate_genes_sync(&mut rng);

            // Determine max captures
            let max_captures = match threat_class {
                1 => 10,
                2 => 5,
                3 => 3,
                4 => 2,
                5 => 1,
                _ => 5,
            };

            (element, threat_class, spawn_lat, spawn_lng, geohash, species_id, genes, max_captures, duration)
        };

        // Now we can await - all random generation is done
        let titan = sqlx::query_as::<_, TitanSpawn>(
            r#"
            INSERT INTO titan_spawns 
            (poi_id, location_lat, location_lng, geohash, element, threat_class, 
             species_id, genes, expires_at, max_captures)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(poi.id)
        .bind(spawn_lat)
        .bind(spawn_lng)
        .bind(&geohash)
        .bind(element)
        .bind(threat_class)
        .bind(species_id)
        .bind(&genes)
        .bind(Utc::now() + duration)
        .bind(max_captures)
        .fetch_one(&self.db.pg)
        .await?;

        tracing::info!(
            "Spawned Titan {} at POI {} ({:?}, Class {})",
            titan.id,
            poi.name,
            element,
            threat_class
        );

        Ok(titan)
    }

    /// Determine element based on terrain (sync version with external rng)
    fn determine_element_sync(&self, terrain: TerrainType, rng: &mut impl Rng) -> Element {
        let roll = rng.gen::<f64>() * 100.0;

        match terrain {
            TerrainType::Water => {
                if roll < 70.0 {
                    Element::Abyssal
                } else if roll < 90.0 {
                    Element::Storm
                } else {
                    Element::Parasitic
                }
            }
            TerrainType::Mountain => {
                if roll < 60.0 {
                    Element::Volcanic
                } else if roll < 85.0 {
                    Element::Storm
                } else {
                    Element::Ossified
                }
            }
            TerrainType::Urban => {
                if roll < 40.0 {
                    Element::Storm
                } else if roll < 75.0 {
                    Element::Void
                } else {
                    Element::Parasitic
                }
            }
            TerrainType::Forest => {
                if roll < 65.0 {
                    Element::Parasitic
                } else if roll < 85.0 {
                    Element::Ossified
                } else {
                    Element::Abyssal
                }
            }
            TerrainType::Desert => {
                if roll < 50.0 {
                    Element::Volcanic
                } else if roll < 85.0 {
                    Element::Ossified
                } else {
                    Element::Void
                }
            }
            TerrainType::Coastal => {
                if roll < 45.0 {
                    Element::Abyssal
                } else if roll < 80.0 {
                    Element::Storm
                } else {
                    Element::Volcanic
                }
            }
            TerrainType::Arctic => {
                if roll < 60.0 {
                    Element::Ossified
                } else if roll < 85.0 {
                    Element::Void
                } else {
                    Element::Storm
                }
            }
        }
    }

    /// Determine threat class based on POI category (sync version)
    fn determine_threat_class_sync(&self, poi: &POI, rng: &mut impl Rng) -> i16 {
        let roll = rng.gen::<f64>() * 100.0;

        // Base distribution
        let mut weights = [60.0, 25.0, 10.0, 4.0, 1.0]; // Class I-V

        // Adjust based on POI spawn weight (higher weight = better chances)
        if poi.spawn_weight >= 4.0 {
            weights[2] *= 2.0; // Class III
            weights[3] *= 3.0; // Class IV
            weights[4] *= 5.0; // Class V
        } else if poi.spawn_weight >= 3.0 {
            weights[2] *= 1.5;
            weights[3] *= 2.0;
        }

        let total: f64 = weights.iter().sum();
        let normalized: Vec<f64> = weights.iter().map(|w| w / total * 100.0).collect();

        let mut cumulative = 0.0;
        for (i, &weight) in normalized.iter().enumerate() {
            cumulative += weight;
            if roll < cumulative {
                return (i + 1) as i16;
            }
        }

        1 // Default to Class I
    }

    /// Generate species ID (sync version)
    fn generate_species_id_sync(&self, element: Element, threat_class: i16, rng: &mut impl Rng) -> i32 {
        let element_base = element.as_u8() as i32 * 1000;
        let class_offset = (threat_class as i32 - 1) * 100;
        let variant = rng.gen_range(1..=10);

        element_base + class_offset + variant
    }

    /// Generate random gene sequence (sync version)
    fn generate_genes_sync(&self, rng: &mut impl Rng) -> Vec<u8> {
        (0..6).map(|_| rng.gen::<u8>()).collect()
    }
}

// Helper trait for hour
trait DateTimeHour {
    fn hour(&self) -> u32;
}

impl DateTimeHour for chrono::DateTime<Utc> {
    fn hour(&self) -> u32 {
        chrono::Timelike::hour(self)
    }
}
