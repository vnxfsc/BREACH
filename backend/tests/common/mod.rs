//! Common test utilities and fixtures

use breach_backend::config::AppConfig;

/// Create a test configuration
pub fn test_config() -> AppConfig {
    AppConfig::default()
}

/// Generate a random test wallet address
pub fn random_wallet_address() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    bs58::encode(bytes).into_string()
}

/// Generate a test JWT token
pub fn test_jwt_token(player_id: uuid::Uuid, wallet: &str) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use chrono::{Duration, Utc};
    
    #[derive(serde::Serialize)]
    struct Claims {
        player_id: uuid::Uuid,
        wallet_address: String,
        exp: i64,
    }
    
    let expiry = Utc::now() + Duration::hours(24);
    let claims = Claims {
        player_id,
        wallet_address: wallet.to_string(),
        exp: expiry.timestamp(),
    };
    
    let key = EncodingKey::from_secret(b"test_secret_key_for_integration_testing");
    encode(&Header::default(), &claims, &key).unwrap()
}
