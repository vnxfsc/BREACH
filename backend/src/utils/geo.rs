//! Geographic utilities

/// Earth radius in meters
pub const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Calculate distance between two points using Haversine formula
pub fn haversine_distance(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lng = (lng2 - lng1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS_M * c
}

/// Calculate bearing between two points
pub fn bearing(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lng = (lng2 - lng1).to_radians();

    let x = delta_lng.sin() * lat2_rad.cos();
    let y = lat1_rad.cos() * lat2_rad.sin() - lat1_rad.sin() * lat2_rad.cos() * delta_lng.cos();

    x.atan2(y).to_degrees()
}

/// Calculate destination point given start point, bearing, and distance
pub fn destination_point(lat: f64, lng: f64, bearing_deg: f64, distance_m: f64) -> (f64, f64) {
    let lat_rad = lat.to_radians();
    let lng_rad = lng.to_radians();
    let bearing_rad = bearing_deg.to_radians();
    let angular_distance = distance_m / EARTH_RADIUS_M;

    let dest_lat = (lat_rad.sin() * angular_distance.cos()
        + lat_rad.cos() * angular_distance.sin() * bearing_rad.cos())
    .asin();

    let dest_lng = lng_rad
        + (bearing_rad.sin() * angular_distance.sin() * lat_rad.cos())
            .atan2(angular_distance.cos() - lat_rad.sin() * dest_lat.sin());

    (dest_lat.to_degrees(), dest_lng.to_degrees())
}

/// Generate a random point within a circle
pub fn random_point_in_circle(center_lat: f64, center_lng: f64, radius_m: f64) -> (f64, f64) {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    // Random distance (sqrt for uniform distribution)
    let distance = radius_m * rng.gen::<f64>().sqrt();

    // Random bearing
    let bearing = rng.gen::<f64>() * 360.0;

    destination_point(center_lat, center_lng, bearing, distance)
}

/// Get geohash neighbors
pub fn get_geohash_neighbors(geohash: &str) -> Vec<String> {
    match geohash::neighbors(geohash) {
        Ok(n) => vec![
            geohash.to_string(),
            n.n,
            n.ne,
            n.e,
            n.se,
            n.s,
            n.sw,
            n.w,
            n.nw,
        ],
        Err(_) => vec![geohash.to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Haversine Distance Tests
    // ========================================

    #[test]
    fn test_haversine_distance_tokyo_osaka() {
        // Tokyo to Osaka is approximately 400km
        let tokyo = (35.6762, 139.6503);
        let osaka = (34.6937, 135.5023);

        let distance = haversine_distance(tokyo.0, tokyo.1, osaka.0, osaka.1);

        // Should be around 400km (actual: ~397km)
        assert!(distance > 390_000.0 && distance < 410_000.0);
    }

    #[test]
    fn test_haversine_distance_same_point() {
        let point = (40.7128, -74.0060); // New York
        let distance = haversine_distance(point.0, point.1, point.0, point.1);
        assert!(distance < 0.001, "Same point should have zero distance");
    }

    #[test]
    fn test_haversine_distance_antipodal() {
        // Approximately antipodal points
        let point1 = (0.0, 0.0);
        let point2 = (0.0, 180.0);
        let distance = haversine_distance(point1.0, point1.1, point2.0, point2.1);
        
        // Half Earth circumference ~20,000km
        let expected = std::f64::consts::PI * EARTH_RADIUS_M;
        assert!((distance - expected).abs() < 1000.0);
    }

    #[test]
    fn test_haversine_distance_short() {
        // Two points 100m apart (approximately)
        let lat1 = 35.6762;
        let lng1 = 139.6503;
        let lat2 = 35.6763; // ~0.0001 degrees = ~11m
        let lng2 = 139.6503;
        
        let distance = haversine_distance(lat1, lng1, lat2, lng2);
        assert!(distance > 10.0 && distance < 15.0, "Expected ~11m, got {}", distance);
    }

    #[test]
    fn test_haversine_distance_symmetric() {
        let point1 = (35.6762, 139.6503);
        let point2 = (34.6937, 135.5023);
        
        let d1 = haversine_distance(point1.0, point1.1, point2.0, point2.1);
        let d2 = haversine_distance(point2.0, point2.1, point1.0, point1.1);
        
        assert!((d1 - d2).abs() < 0.001, "Distance should be symmetric");
    }

    // ========================================
    // Bearing Tests
    // ========================================

    #[test]
    fn test_bearing_north() {
        let from = (35.0, 139.0);
        let to = (36.0, 139.0); // Due north
        
        let b = bearing(from.0, from.1, to.0, to.1);
        assert!((b - 0.0).abs() < 1.0, "Expected ~0 degrees (north), got {}", b);
    }

    #[test]
    fn test_bearing_east() {
        let from = (0.0, 0.0);
        let to = (0.0, 1.0); // Due east on equator
        
        let b = bearing(from.0, from.1, to.0, to.1);
        assert!((b - 90.0).abs() < 1.0, "Expected ~90 degrees (east), got {}", b);
    }

    #[test]
    fn test_bearing_south() {
        let from = (36.0, 139.0);
        let to = (35.0, 139.0); // Due south
        
        let b = bearing(from.0, from.1, to.0, to.1);
        assert!((b - 180.0).abs() < 1.0 || (b + 180.0).abs() < 1.0, 
                "Expected ~180 degrees (south), got {}", b);
    }

    #[test]
    fn test_bearing_west() {
        let from = (0.0, 1.0);
        let to = (0.0, 0.0); // Due west on equator
        
        let b = bearing(from.0, from.1, to.0, to.1);
        assert!((b - (-90.0)).abs() < 1.0 || (b - 270.0).abs() < 1.0, 
                "Expected ~-90 or 270 degrees (west), got {}", b);
    }

    // ========================================
    // Destination Point Tests
    // ========================================

    #[test]
    fn test_destination_point_north() {
        let start = (35.0, 139.0);
        let (lat, lng) = destination_point(start.0, start.1, 0.0, 1000.0); // 1km north
        
        assert!(lat > start.0, "Should move north (higher latitude)");
        assert!((lng - start.1).abs() < 0.001, "Longitude should stay same");
    }

    #[test]
    fn test_destination_point_east() {
        let start = (0.0, 0.0); // Equator
        let (lat, lng) = destination_point(start.0, start.1, 90.0, 1000.0); // 1km east
        
        assert!(lat.abs() < 0.001, "Latitude should stay ~0 at equator");
        assert!(lng > start.1, "Should move east (higher longitude)");
    }

    #[test]
    fn test_destination_point_roundtrip() {
        let start = (35.6762, 139.6503);
        let distance = 5000.0; // 5km
        let bearing_deg = 45.0;
        
        let (mid_lat, mid_lng) = destination_point(start.0, start.1, bearing_deg, distance);
        
        // Verify distance
        let calculated_dist = haversine_distance(start.0, start.1, mid_lat, mid_lng);
        assert!((calculated_dist - distance).abs() < 10.0, 
                "Distance should be ~5000m, got {}", calculated_dist);
    }

    // ========================================
    // Random Point in Circle Tests
    // ========================================

    #[test]
    fn test_random_point_in_circle_within_bounds() {
        let center = (35.6762, 139.6503);
        let radius = 100.0;

        for _ in 0..100 {
            let (lat, lng) = random_point_in_circle(center.0, center.1, radius);
            let distance = haversine_distance(center.0, center.1, lat, lng);
            assert!(distance <= radius + 1.0, 
                    "Point at distance {} should be within radius {}", distance, radius);
        }
    }

    #[test]
    fn test_random_point_in_circle_distribution() {
        let center = (35.6762, 139.6503);
        let radius = 1000.0;
        let mut distances: Vec<f64> = Vec::new();
        
        for _ in 0..1000 {
            let (lat, lng) = random_point_in_circle(center.0, center.1, radius);
            let distance = haversine_distance(center.0, center.1, lat, lng);
            distances.push(distance);
        }
        
        // Check that points are distributed across the circle
        let inner_count = distances.iter().filter(|&&d| d < radius / 2.0).count();
        let outer_count = distances.iter().filter(|&&d| d >= radius / 2.0).count();
        
        // For uniform distribution, outer ring (50-100%) has 3x area of inner (0-50%)
        // So we expect ~75% in outer, ~25% in inner
        let inner_ratio = inner_count as f64 / distances.len() as f64;
        assert!(inner_ratio > 0.15 && inner_ratio < 0.40, 
                "Inner ratio {} should be ~0.25 for uniform distribution", inner_ratio);
    }

    // ========================================
    // Geohash Neighbors Tests
    // ========================================

    #[test]
    fn test_geohash_neighbors_valid() {
        let geohash = "xn77h"; // Tokyo area
        let neighbors = get_geohash_neighbors(geohash);
        
        assert_eq!(neighbors.len(), 9, "Should return center + 8 neighbors");
        assert!(neighbors.contains(&geohash.to_string()), "Should include center");
    }

    #[test]
    fn test_geohash_neighbors_short() {
        let geohash = "x";
        let neighbors = get_geohash_neighbors(geohash);
        
        assert!(!neighbors.is_empty(), "Should return at least the center");
    }

    #[test]
    fn test_geohash_neighbors_invalid() {
        // Note: Empty string causes panic in geohash crate, so we test with invalid but non-empty
        let geohash = "a"; // Single character - edge case
        let neighbors = get_geohash_neighbors(geohash);
        
        // Should gracefully handle and return at least the input
        assert!(!neighbors.is_empty());
    }

    #[test]
    fn test_geohash_neighbors_unique() {
        let geohash = "xn77h";
        let neighbors = get_geohash_neighbors(geohash);
        
        let mut unique: Vec<String> = neighbors.clone();
        unique.sort();
        unique.dedup();
        
        assert_eq!(neighbors.len(), unique.len(), "All neighbors should be unique");
    }

    // ========================================
    // Edge Cases
    // ========================================

    #[test]
    fn test_poles() {
        // North pole
        let north_pole = (90.0, 0.0);
        let south_pole = (-90.0, 0.0);
        
        let distance = haversine_distance(north_pole.0, north_pole.1, south_pole.0, south_pole.1);
        
        // Should be ~20,000km (half circumference)
        let expected = std::f64::consts::PI * EARTH_RADIUS_M;
        assert!((distance - expected).abs() < 1000.0);
    }

    #[test]
    fn test_date_line_crossing() {
        // Points on either side of the date line
        let west = (0.0, 179.0);
        let east = (0.0, -179.0);
        
        let distance = haversine_distance(west.0, west.1, east.0, east.1);
        
        // Should be ~222km (2 degrees at equator), not ~40,000km
        assert!(distance < 500_000.0, "Date line crossing should be short distance");
    }
}
