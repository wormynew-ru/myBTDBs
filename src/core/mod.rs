pub mod events;
pub mod input;
pub mod state;
pub mod types;

use bevy::prelude::*;

use events::{
    ActivateAbilityEvent, BloonLeakedEvent, BloonPoppedEvent, BuildTowerEvent,
    ChangePriorityEvent, SellTowerEvent, SendBloonPackEvent, SpawnBloonEvent, UpgradeTowerEvent,
};
use input::{update_pointer_input, PointerInput};
use state::{BuildSelection, GameState, MatchResult, MatchSettings, SelectedTower};
use types::SidesResource;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<MatchSettings>()
            .init_resource::<MatchResult>()
            .init_resource::<SelectedTower>()
            .init_resource::<BuildSelection>()
            .init_resource::<SidesResource>()
            .init_resource::<PointerInput>()
            .add_event::<SpawnBloonEvent>()
            .add_event::<BuildTowerEvent>()
            .add_event::<UpgradeTowerEvent>()
            .add_event::<SellTowerEvent>()
            .add_event::<ChangePriorityEvent>()
            .add_event::<ActivateAbilityEvent>()
            .add_event::<SendBloonPackEvent>()
            .add_event::<BloonLeakedEvent>()
            .add_event::<BloonPoppedEvent>()
            .add_systems(Update, update_pointer_input);
    }
}
