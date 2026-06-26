use bevy::prelude::*;

impl Component for crate::nbody::System {
    const STORAGE_TYPE: bevy::ecs::component::StorageType
        = bevy::ecs::component::StorageType::Table;

    type Mutability = bevy::ecs::component::Mutable;
}
impl Resource for crate::nbody::System {}