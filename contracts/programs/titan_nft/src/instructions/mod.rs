//! Instruction handlers

pub mod add_experience;
pub mod evolve;
pub mod fuse;
pub mod initialize;
pub mod level_up;
pub mod mint_titan;
pub mod set_paused;
pub mod transfer;
pub mod update_config;

/// Instruction discriminators
pub mod discriminator {
    pub const INITIALIZE: u8 = 0;
    pub const MINT_TITAN: u8 = 1;
    pub const LEVEL_UP: u8 = 2;
    pub const EVOLVE: u8 = 3;
    pub const FUSE: u8 = 4;
    pub const TRANSFER: u8 = 5;
    pub const UPDATE_CONFIG: u8 = 6;
    pub const SET_PAUSED: u8 = 7;
    pub const ADD_EXPERIENCE: u8 = 8;
}
