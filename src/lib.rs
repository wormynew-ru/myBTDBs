pub mod core;
pub mod data;
pub mod plugins;
pub mod ui;
pub mod utils;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::{WindowPlugin, WindowResolution};

use core::CorePlugin;
use plugins::GamePlugins;
use ui::UiPlugin;

#[bevy_main]
pub fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "myBTDBs — Bloons TD Battles PvE".to_string(),
                    resolution: WindowResolution::new(1280.0, 720.0),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins((CorePlugin, GamePlugins, UiPlugin))
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1280.0,
                min_height: 720.0,
            },
            ..OrthographicProjection::default_2d()
        },
    ));
}
