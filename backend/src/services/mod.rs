//! Business logic services

pub mod auth;
mod achievement;
mod battle;
mod capture;
mod chat;
mod friend;
mod guild;
mod inventory;
mod leaderboard;
mod location;
mod map;
mod marketplace;
mod notification;
mod player;
mod pvp;
mod quest;
mod spawn;

pub use achievement::AchievementService;
pub use auth::AuthService;
pub use battle::BattleService;
pub use capture::CaptureService;
pub use chat::ChatService;
pub use friend::FriendService;
pub use guild::GuildService;
pub use inventory::InventoryService;
pub use leaderboard::LeaderboardService;
pub use location::LocationService;
pub use map::MapService;
pub use marketplace::MarketplaceService;
pub use notification::NotificationService;
pub use player::PlayerService;
pub use pvp::PvpService;
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
    pub chat: ChatService,
    pub friend: FriendService,
    pub guild: GuildService,
    pub inventory: InventoryService,
    pub leaderboard: LeaderboardService,
    pub location: LocationService,
    pub map: MapService,
    pub marketplace: MarketplaceService,
    pub notification: NotificationService,
    pub player: PlayerService,
    pub pvp: PvpService,
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
            chat: ChatService::new(db.clone()),
            friend: FriendService::new(db.clone()),
            guild: GuildService::new(db.clone()),
            inventory: InventoryService::new(db.clone()),
            leaderboard: LeaderboardService::new(db.clone()),
            location: LocationService::new(config.clone(), db.clone()),
            map: MapService::new(db.clone()),
            marketplace: MarketplaceService::new(db.clone()),
            notification: NotificationService::new(db.clone()),
            player: PlayerService::new(db.clone()),
            pvp: PvpService::new(db.clone()),
            quest: QuestService::new(db.clone()),
            spawn: SpawnService::new(config.clone(), db.clone()),
        }
    }
}
