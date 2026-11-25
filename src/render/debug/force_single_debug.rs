use bevy::prelude::*;
use crate::sim::{
    Simulator as _,
    SimulatorExt as _
};
use super::*;

#[derive(Resource)]
pub struct ForceSingleDebugState {
    curr_body: usize,
}

#[derive(Component)]
pub struct ForceSingleTag { body_id: usize }

pub fn setup_force_single(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    mut nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
) {
    println!("Setup Force Single Debug");

    commands.insert_resource(ForceSingleDebugState {
        curr_body: 0,
    });

    let circle = &render_data.circle_mesh;
    let segment = &render_data.segment_mesh;
    let red = &render_data.red_material;
    let mat = &render_data.force_material;

    nbody.pull_updates();
    for (body_id, body, forces) in nbody.get_bodies().iter().enumerate()
        .map(|(body_id, body)| (body_id, body, nbody.get_body_forces(body_id)))
    {
        let Some(forces) = forces else { continue; };

        let min_force = *forces.iter().map(|f| ordered_float::OrderedFloat(f.force.magnitude())).min().unwrap();
        let max_force = *forces.iter().map(|f| ordered_float::OrderedFloat(f.force.magnitude())).max().unwrap();
        let diff_force = max_force - min_force;

        let scale = body.mass.sqrt() * 5.0;
        commands.spawn((
            Visibility::Visible,
            Mesh2d(circle.to_owned()),
            MeshMaterial2d(red.to_owned()),
            Transform::from_translation([body.position.x, body.position.y, 0.0].into())
                .with_scale([scale, scale, 1.0].into()),
            ForceSingleTag { body_id },
            SimEntityTag,
        ));

        for sim::SingleBodyForce { com, force } in forces.into_iter() {
            let magnitude = (force.magnitude() - min_force) / diff_force;

            commands.spawn((
                Visibility::Visible,
                Mesh2d(segment.to_owned()),
                MeshMaterial2d(mat.to_owned()),
                Transform::from_translation([com.x, com.y, 0.0].into()),
                ForceSingleTag { body_id },
                SimEntityTag,
            ))
            .with_child((
                Mesh2d(segment.to_owned()),
                MeshMaterial2d(mat.to_owned()),
                Transform::from_rotation(Quat::from_rotation_arc_2d(Vec2::Y, Vec2::new(force.x, force.y).normalize()))
                    .with_scale([1.0, 20.0 * magnitude + 10.0, 1.0].into()),
            ))
            .with_child((
                Mesh2d(circle.to_owned()),
                MeshMaterial2d(mat.to_owned()),
                Transform::from_scale([magnitude + 2.0, magnitude + 2.0, 1.0].into()),
            ));
        }
    }
}

pub fn process_inputs_debug_force_single(
    keys: Res<ButtonInput<bevy::input::keyboard::Key>>,
    mut debug_data: ResMut<ForceSingleDebugState>,
    nbody: Res<sim::async_simulator::AsyncSimulatorFrontend>,
    force_single_items: Query<(&mut Visibility, &ForceSingleTag)>,
) {
    if keys.pressed(bevy::input::keyboard::Key::ArrowUp) {
        debug_data.curr_body =
            if debug_data.curr_body == nbody.get_bodies().len() - 1 {
                0
            } else {
                debug_data.curr_body + 1
            };
    }

    if keys.pressed(bevy::input::keyboard::Key::ArrowDown) {
        debug_data.curr_body =
            if debug_data.curr_body == 0 {
                nbody.get_bodies().len() - 1
            } else {
                debug_data.curr_body - 1
            };
    }

    for (mut vis, item) in force_single_items {
        if item.body_id == debug_data.curr_body {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

pub fn destroy_force_single(
    mut commands: Commands,
    quadtree_items: Query<Entity, With<ForceSingleTag>>,
) {
    for entity in quadtree_items {
        commands.entity(entity).despawn();
    }
}
