use bevy::prelude::*;

impl Component for crate::sim::Simulator {
    const STORAGE_TYPE: bevy::ecs::component::StorageType
        = bevy::ecs::component::StorageType::Table;

    type Mutability = bevy::ecs::component::Mutable;
}
impl Resource for crate::sim::Simulator {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum SimState {
    #[default]
    Simulate,
    Debug,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(SimState = SimState::Debug)]
pub enum DebugState {
    #[default]
    QuadTree,
    Forces,
    ForceSingle,
}