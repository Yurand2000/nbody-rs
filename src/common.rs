use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum GameSceneState {
    #[default]
    Loading,
    MainMenu,
    Game,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum SimulationState {
    #[default]
    Unloaded,
    Loading,
    Simulation
}

#[derive(Resource)]
pub enum SceneToLoad {
    Orbit
}

#[derive(Component)]
pub struct BodyId(pub crate::sim::BodyId);
