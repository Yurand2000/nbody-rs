use bevy::prelude::*;
use crate::sim::{
    Simulator as _,
    SimulatorExt as _
};
use super::*;

#[derive(Component)]
pub struct ForceTag;

pub fn setup_forces(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    mut nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
) {
    println!("Setup Forces Debug");

    let Some(forces) = nbody.get_forces() else { return; };

    let segment = &render_data.segment_mesh;
    let mat = &render_data.force_material;

    let min_force = *forces.iter().map(|f| ordered_float::OrderedFloat(f.magnitude())).min().unwrap();
    let max_force = *forces.iter().map(|f| ordered_float::OrderedFloat(f.magnitude())).max().unwrap();
    let diff_force = max_force - min_force;

    nbody.pull_updates();
    for (body, force) in nbody.get_bodies().iter().zip(forces.into_iter()) {
        let magnitude = 20.0 * (force.magnitude() - min_force) / diff_force + 10.0;

        commands.spawn((
            Mesh2d(segment.to_owned()),
            MeshMaterial2d(mat.to_owned()),
            Transform::from_translation([body.position.x, body.position.y, 0.0].into())
                .with_rotation(Quat::from_rotation_arc_2d(Vec2::Y, Vec2::new(force.x, force.y).normalize()))
                .with_scale([1.0, magnitude, 1.0].into()),
            ForceTag,
            SimEntityTag,
        ));
    }
}

pub fn destroy_forces(
    mut commands: Commands,
    quadtree_items: Query<Entity, With<ForceTag>>
) {
    for entity in quadtree_items {
        commands.entity(entity).despawn();
    }
}
