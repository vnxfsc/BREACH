//! Map and spatial query service

use uuid::Uuid;

use crate::db::Database;
use crate::error::ApiResult;
use crate::models::{GeoPoint, POI, POIResponse, TitanSpawn, TitanSpawnResponse};

/// Map service for spatial queries
#[derive(Clone)]
pub struct MapService {
    db: Database,
}

impl MapService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get nearby Titans within a radius
    pub async fn get_nearby_titans(
        &self,
        lat: f64,
        lng: f64,
        radius_meters: f64,
    ) -> ApiResult<Vec<TitanSpawnResponse>> {
        // Generate geohash prefix for efficient querying
        // Use 5-char precision for broader coverage (~5km cells)
        let geohash = geohash::encode(
            geohash::Coord { x: lng, y: lat },
            5, // 5-char precision ~5km
        )
        .unwrap_or_default();

        // Get neighbors for edge cases
        let mut geohash_prefixes: Vec<String> = vec![geohash.clone()];
        if let Ok(neighbors) = geohash::neighbors(&geohash) {
            geohash_prefixes.extend(vec![
                neighbors.n,
                neighbors.ne,
                neighbors.e,
                neighbors.se,
                neighbors.s,
                neighbors.sw,
                neighbors.w,
                neighbors.nw,
            ]);
        }
        
        let patterns: Vec<String> = geohash_prefixes
            .iter()
            .map(|g| format!("{}%", g))
            .collect();
        
        tracing::debug!("Searching titans: lat={}, lng={}, geohash={}, patterns={:?}", lat, lng, geohash, patterns);

        // Query database - use simple OR conditions for better compatibility
        let titans = sqlx::query_as::<_, TitanSpawn>(
            r#"
            SELECT t.id, t.poi_id, t.location_lat, t.location_lng, t.geohash,
                   t.element, t.threat_class, t.species_id, t.genes,
                   t.spawned_at, t.expires_at, t.captured_by, t.captured_at,
                   t.capture_count, t.max_captures
            FROM titan_spawns t
            WHERE t.expires_at > NOW()
              AND (t.captured_by IS NULL OR t.capture_count < t.max_captures)
              AND ST_DWithin(
                ST_SetSRID(ST_MakePoint(t.location_lng, t.location_lat), 4326)::geography,
                ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography,
                $3
              )
            ORDER BY ST_Distance(
                ST_SetSRID(ST_MakePoint(t.location_lng, t.location_lat), 4326)::geography,
                ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography
            )
            LIMIT 50
            "#,
        )
        .bind(lng)
        .bind(lat)
        .bind(radius_meters)
        .fetch_all(&self.db.pg)
        .await?;

        tracing::info!("Query returned {} titans for params: lng={}, lat={}, radius={}", 
            titans.len(), lng, lat, radius_meters);

        // Convert to response format
        let responses: Vec<TitanSpawnResponse> = titans
            .into_iter()
            .map(|t| {
                let distance = crate::services::location::haversine_distance(
                    lat, lng, t.location_lat, t.location_lng,
                );

                TitanSpawnResponse {
                    id: t.id,
                    location: GeoPoint {
                        lat: t.location_lat,
                        lng: t.location_lng,
                    },
                    element: t.element,
                    threat_class: t.threat_class,
                    species_id: t.species_id,
                    distance: Some(distance),
                    expires_at: t.expires_at,
                    poi_name: None, // Would need JOIN to get this
                    is_available: t.captured_by.is_none() || t.capture_count < t.max_captures,
                }
            })
            .collect();

        Ok(responses)
    }

    /// Get POIs in a bounding box
    pub async fn get_pois_in_bounds(
        &self,
        sw_lat: f64,
        sw_lng: f64,
        ne_lat: f64,
        ne_lng: f64,
    ) -> ApiResult<Vec<POIResponse>> {
        let pois = sqlx::query_as::<_, POI>(
            r#"
            SELECT * FROM pois
            WHERE location_lat BETWEEN $1 AND $3
              AND location_lng BETWEEN $2 AND $4
              AND is_active = true
            LIMIT 100
            "#,
        )
        .bind(sw_lat)
        .bind(sw_lng)
        .bind(ne_lat)
        .bind(ne_lng)
        .fetch_all(&self.db.pg)
        .await?;

        // Check which POIs have active Titans
        let poi_ids: Vec<Uuid> = pois.iter().map(|p| p.id).collect();

        let active_poi_ids: Vec<Uuid> = sqlx::query_scalar(
            r#"
            SELECT DISTINCT poi_id FROM titan_spawns
            WHERE poi_id = ANY($1)
              AND expires_at > NOW()
              AND (captured_by IS NULL OR capture_count < max_captures)
            "#,
        )
        .bind(&poi_ids)
        .fetch_all(&self.db.pg)
        .await?;

        let responses: Vec<POIResponse> = pois
            .into_iter()
            .map(|p| POIResponse {
                id: p.id,
                name: p.name,
                category: p.category,
                location: GeoPoint {
                    lat: p.location_lat,
                    lng: p.location_lng,
                },
                has_active_titan: active_poi_ids.contains(&p.id),
                terrain_type: p.terrain_type,
            })
            .collect();

        Ok(responses)
    }

    /// Get a single Titan by ID
    pub async fn get_titan(&self, titan_id: Uuid) -> ApiResult<Option<TitanSpawn>> {
        let titan = sqlx::query_as::<_, TitanSpawn>(
            r#"
            SELECT * FROM titan_spawns WHERE id = $1
            "#,
        )
        .bind(titan_id)
        .fetch_optional(&self.db.pg)
        .await?;

        Ok(titan)
    }
}
