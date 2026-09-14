use bevy::prelude::*;

pub mod common;

#[cfg(feature = "cuda")]
pub mod cuda;
pub mod sim;
pub mod render;

pub mod menu;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(DefaultPlugins)
            .init_state::<common::GameSceneState>()
            .init_state::<common::SimulationState>()

            .add_systems(Startup, game_setup)
            .add_plugins(menu::MainMenuPlugin)
            .add_plugins(render::SceneRenderPlugin)
            .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        ;
    }
}

fn game_setup(
    mut commands: Commands,
) {
    commands.insert_resource(Time::<Fixed>::from_hz(30.0));
}
