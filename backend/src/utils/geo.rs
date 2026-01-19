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

    #[test]
    fn test_haversine_distance() {
        // Tokyo to Osaka is approximately 400km
        let tokyo = (35.6762, 139.6503);
        let osaka = (34.6937, 135.5023);

        let distance = haversine_distance(tokyo.0, tokyo.1, osaka.0, osaka.1);

        // Should be around 400km
        assert!(distance > 390_000.0 && distance < 410_000.0);
    }

    #[test]
    fn test_random_point_in_circle() {
        let center = (35.6762, 139.6503);
        let radius = 100.0;

        for _ in 0..100 {
            let (lat, lng) = random_point_in_circle(center.0, center.1, radius);
            let distance = haversine_distance(center.0, center.1, lat, lng);
            assert!(distance <= radius + 1.0); // Small tolerance for floating point
        }
    }
}
