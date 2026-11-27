use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Player,
    Bot,
}

impl Side {
    pub fn opponent(self) -> Self {
        match self {
            Side::Player => Side::Bot,
            Side::Bot => Side::Player,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Position(pub Vec2);

#[derive(Component)]
pub struct MatchEntity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BloonLayer {
    Red,
    Blue,
    Green,
    Yellow,
    Pink,
    Rainbow,
    Ceramic,
    Moab,
    Bfb,
    Zomg,
}

impl BloonLayer {
    pub fn is_moab_class(self) -> bool {
        matches!(self, BloonLayer::Moab | BloonLayer::Bfb | BloonLayer::Zomg)
    }

    pub fn is_zomg(self) -> bool {
        self == BloonLayer::Zomg
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TowerType {
    DartMonkey,
    Cannon,
    IceTower,
    NinjaMonkey,
    Bloonchipper,
    Village,
    SpikeFactory,
    Commander,
    MilitaryBase,
    CrookBoss,
    Farm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TargetPriority {
    #[default]
    First,
    Last,
    Close,
    Strong,
}

impl TargetPriority {
    pub fn label(self) -> &'static str {
        match self {
            TargetPriority::First => "First",
            TargetPriority::Last => "Last",
            TargetPriority::Close => "Close",
            TargetPriority::Strong => "Strong",
        }
    }

    pub fn next(self) -> Self {
        match self {
            TargetPriority::First => TargetPriority::Last,
            TargetPriority::Last => TargetPriority::Close,
            TargetPriority::Close => TargetPriority::Strong,
            TargetPriority::Strong => TargetPriority::First,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SideEconomy {
    pub lives: i32,
    pub cash: i32,
    pub eco: i32,
    pub pops: u32,
}

impl Default for SideEconomy {
    fn default() -> Self {
        Self {
            lives: 150,
            cash: 650,
            eco: 250,
            pops: 0,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct SidesResource {
    pub player: SideEconomy,
    pub bot: SideEconomy,
    pub eco_timer: f32,
    pub wave: u32,
    pub wave_timer: f32,
    pub match_time: f32,
}

impl SidesResource {
    pub fn get(&self, side: Side) -> &SideEconomy {
        match side {
            Side::Player => &self.player,
            Side::Bot => &self.bot,
        }
    }

    pub fn get_mut(&mut self, side: Side) -> &mut SideEconomy {
        match side {
            Side::Player => &mut self.player,
            Side::Bot => &mut self.bot,
        }
    }
}
