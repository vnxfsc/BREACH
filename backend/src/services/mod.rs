//! Business logic services

pub mod auth;
mod achievement;
mod battle;
mod capture;
mod inventory;
mod leaderboard;
mod location;
mod map;
mod player;
mod quest;
mod spawn;

pub use achievement::AchievementService;
pub use auth::AuthService;
pub use battle::BattleService;
pub use capture::CaptureService;
pub use inventory::InventoryService;
pub use leaderboard::LeaderboardService;
pub use location::LocationService;
pub use map::MapService;
pub use player::PlayerService;
pub use quest::QuestService;
pub use spawn::SpawnService;

use crate::config::AppConfig;
use crate::db::Database;

/// Container for all services
#[derive(Clone)]
pub struct Services {
    pub auth: AuthService,
    pub achievement: AchievementService,
    pub battle: BattleService,
    pub capture: CaptureService,
    pub inventory: InventoryService,
    pub leaderboard: LeaderboardService,
    pub location: LocationService,
    pub map: MapService,
    pub player: PlayerService,
    pub quest: QuestService,
    pub spawn: SpawnService,
}

impl Services {
    pub fn new(config: &AppConfig, db: Database) -> Self {
        Self {
            auth: AuthService::new(config.clone()),
            achievement: AchievementService::new(db.clone()),
            battle: BattleService::new(db.clone()),
            capture: CaptureService::new(config.clone(), db.clone()),
            inventory: InventoryService::new(db.clone()),
            leaderboard: LeaderboardService::new(db.clone()),
            location: LocationService::new(config.clone(), db.clone()),
            map: MapService::new(db.clone()),
            player: PlayerService::new(db.clone()),
            quest: QuestService::new(db.clone()),
            spawn: SpawnService::new(config.clone(), db.clone()),
        }
    }
}
