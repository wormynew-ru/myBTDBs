use bevy::prelude::*;

use super::types::{Side, TowerType};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Difficulty {
    Easy,
    #[default]
    Medium,
    Hard,
}

impl Difficulty {
    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }

    pub fn ai_interval(self) -> f32 {
        match self {
            Difficulty::Easy => 1.5,
            Difficulty::Medium => 0.9,
            Difficulty::Hard => 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MapId {
    #[default]
    Meadow,
    Zigzag,
    Canyon,
}

impl MapId {
    pub fn label(self) -> &'static str {
        match self {
            MapId::Meadow => "Meadow Curve",
            MapId::Zigzag => "Twin Zigzag",
            MapId::Canyon => "Spiral Canyon",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameMode {
    #[default]
    Standard,
    Turbo,
}

impl GameMode {
    pub fn label(self) -> &'static str {
        match self {
            GameMode::Standard => "Standard (5)",
            GameMode::Turbo => "Turbo (6)",
        }
    }

    pub fn loadout_size(self) -> usize {
        match self {
            GameMode::Standard => 5,
            GameMode::Turbo => 6,
        }
    }

    pub fn starting_cash(self) -> i32 {
        match self {
            GameMode::Standard => 650,
            GameMode::Turbo => 1500,
        }
    }

    pub fn starting_eco(self) -> i32 {
        match self {
            GameMode::Standard => 250,
            GameMode::Turbo => 400,
        }
    }

    pub fn wave_duration(self) -> f32 {
        match self {
            GameMode::Standard => 16.0,
            GameMode::Turbo => 10.0,
        }
    }

    pub fn eco_interval(self) -> f32 {
        match self {
            GameMode::Standard => 6.0,
            GameMode::Turbo => 4.0,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct MatchSettings {
    pub difficulty: Difficulty,
    pub map_id: MapId,
    pub game_mode: GameMode,
    pub player_loadout: Vec<TowerType>,
    pub bot_loadout: Vec<TowerType>,
    pub speed_mult: f32,
    pub paused: bool,
}

impl Default for MatchSettings {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Medium,
            map_id: MapId::Meadow,
            game_mode: GameMode::Standard,
            player_loadout: vec![
                TowerType::DartMonkey,
                TowerType::Cannon,
                TowerType::NinjaMonkey,
                TowerType::SpikeFactory,
                TowerType::Farm,
            ],
            bot_loadout: Vec::new(),
            speed_mult: 1.0,
            paused: false,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct MatchResult {
    pub winner: Option<Side>,
    pub wave_reached: u32,
    pub match_time: f32,
}

#[derive(Resource, Debug, Default)]
pub struct SelectedTower(pub Option<Entity>);

#[derive(Resource, Debug, Default)]
pub struct BuildSelection(pub Option<TowerType>);
