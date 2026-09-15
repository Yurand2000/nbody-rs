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
pub struct LoadingData {
    pub scene: SceneToLoad,
}

#[derive(Component)]
#[derive(Default, Clone, Copy)]
pub enum SceneToLoad {
    #[default]
    TwoBodies,
    Grid,
    Orbit,
}

#[derive(Component)]
pub struct BodyId(pub crate::sim::BodyId);
