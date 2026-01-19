//! Application configuration management

use serde::Deserialize;

/// Main application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub solana: SolanaConfig,
    pub auth: AuthConfig,
    pub game: GameConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub env: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub titan_program_id: String,
    pub game_program_id: String,
    pub breach_token_mint: String,
    pub backend_keypair_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub signature_expiry_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GameConfig {
    pub capture_radius_meters: f64,
    pub capture_cooldown_seconds: u64,
    pub max_speed_mps: f64,
    pub location_accuracy_threshold: f64,
}

impl AppConfig {
    /// Load configuration from environment and config files
    pub fn load() -> anyhow::Result<Self> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            // Start with default values
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("server.env", "development")?
            .set_default("database.max_connections", 10)?
            .set_default("database.min_connections", 2)?
            .set_default("redis.pool_size", 10)?
            .set_default("auth.jwt_expiry_hours", 24)?
            .set_default("auth.signature_expiry_seconds", 300)?
            .set_default("game.capture_radius_meters", 50.0)?
            .set_default("game.capture_cooldown_seconds", 300)?
            .set_default("game.max_speed_mps", 42.0)?
            .set_default("game.location_accuracy_threshold", 100.0)?
            // Load from config file
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::File::with_name("config/local").required(false))
            // Override with environment variables
            .add_source(
                config::Environment::default()
                    .separator("__")
                    .prefix("BREACH"),
            )
            .build()?;

        let app_config: AppConfig = config.try_deserialize()?;
        Ok(app_config)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                env: "development".to_string(),
            },
            database: DatabaseConfig {
                url: "postgres://localhost/breach".to_string(),
                max_connections: 10,
                min_connections: 2,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                pool_size: 10,
            },
            solana: SolanaConfig {
                rpc_url: "https://api.devnet.solana.com".to_string(),
                ws_url: "wss://api.devnet.solana.com".to_string(),
                titan_program_id: "3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7".to_string(),
                game_program_id: "DLk2GnDu9AYn7PeLprEDHDYH9UWKENX47UqqfeiQBaSX".to_string(),
                breach_token_mint: "CSH2Vz4MbgTLzB9SYJ7gBwNsyu7nKpbvEJzKQLgmmjt4".to_string(),
                backend_keypair_path: "~/.config/solana/backend-keypair.json".to_string(),
            },
            auth: AuthConfig {
                jwt_secret: "development-secret-change-in-production".to_string(),
                jwt_expiry_hours: 24,
                signature_expiry_seconds: 300,
            },
            game: GameConfig {
                capture_radius_meters: 50.0,
                capture_cooldown_seconds: 300,
                max_speed_mps: 42.0,
                location_accuracy_threshold: 100.0,
            },
        }
    }
}
