//! Business logic services

pub mod auth;
mod capture;
mod location;
mod map;
mod player;
mod spawn;

pub use auth::AuthService;
pub use capture::CaptureService;
pub use location::LocationService;
pub use map::MapService;
pub use player::PlayerService;
pub use spawn::SpawnService;

use crate::config::AppConfig;
use crate::db::Database;

/// Container for all services
#[derive(Clone)]
pub struct Services {
    pub auth: AuthService,
    pub capture: CaptureService,
    pub location: LocationService,
    pub map: MapService,
    pub player: PlayerService,
    pub spawn: SpawnService,
}

impl Services {
    pub fn new(config: &AppConfig, db: Database) -> Self {
        Self {
            auth: AuthService::new(config.clone()),
            capture: CaptureService::new(config.clone(), db.clone()),
            location: LocationService::new(config.clone(), db.clone()),
            map: MapService::new(db.clone()),
            player: PlayerService::new(db.clone()),
            spawn: SpawnService::new(config.clone(), db.clone()),
        }
    }
}
