use bevy::prelude::*;
use rand::RngExt as _;
use crate::sim::Simulator as _;
use super::*;

fn new_body(
    mass: f32,
    pos: Vec2,
    vel: Vec2,
    render_data: &MyAssetData,
    commands: &mut Commands,
    nbody: &mut sim::MultiSimulator,
) {
    let body = sim::Body {
        mass,
        position: sim::Vec2f::new(pos.x, pos.y),
        velocity: sim::Vec2f::new(vel.x, vel.y),
    };

    let id = BodyId(nbody.insert_body(body));
    let mass = mass.cbrt();

    commands.spawn((
        id,
        SimEntityTag,
        Mesh2d(render_data.circle_mesh.to_owned()),
        MeshMaterial2d(render_data.white_material.to_owned()),
        Transform::from_xyz(pos.x, pos.y, 0.0)
            .with_scale(Vec3::from([mass, mass, 1.0]))
    ));
}

fn orbit_scenario(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    mut nbody: NonSendMut<sim::MultiSimulator>,
) {
    let g = nbody.get_params().g_constant;
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(17);

    let mut new_body = |mass, pos, vel|
        new_body(mass, pos, vel, &render_data, &mut commands, &mut nbody);

    // central mass
    let sun_mass = 10000.0;
    new_body(sun_mass, Vec2::ZERO, Vec2::ZERO);

    let mut make_ring = |center_mass: f32, center: Vec2, center_vel: Vec2, ring_radius: f32, ring_delta: f32, body_min_mass, body_max_mass, min_bodies, max_bodies|
    {
        let bodies = rng.random_range(min_bodies ..= max_bodies);
        (0 .. bodies).into_iter().map(|_| {
            let mass = rng.random_range(body_min_mass ..= body_max_mass);
            let radius: f32 = ring_radius + rng.random_range(-ring_delta ..= ring_delta);

            let theta = rng.random::<f32>() * std::f32::consts::TAU;
            let dir = Vec2::from_angle(theta);
            let pos = dir * radius + center;

            let vel_mag = f32::sqrt(center_mass * g / radius);
            let vel = Vec2::new(-dir.y, dir.x).normalize() * vel_mag + center_vel;

            new_body(mass, pos, vel);
            (mass, pos, vel)
        })
        .collect::<Vec<_>>()
    };

    make_ring(sun_mass, Vec2::ZERO, Vec2::ZERO, 400.0, 20.0, 0.1, 0.5, 3000, 5000);

    let objs = make_ring(sun_mass, Vec2::ZERO, Vec2::ZERO, 100.0, 50.0, 100.0, 500.0, 3, 4);
    for (obj_mass, obj_pos, obj_vel) in objs.into_iter() {
        make_ring(obj_mass, obj_pos, obj_vel, 50.0, 5.0, 10.0, 20.0, 1, 2);
    }

    // // ring
    // let ring_radius = 400.0;
    // let ring_delta = 10.0;
    // let body_min_mass = 0.1;
    // let body_max_mass = 0.5;
    // let ring_bodies = 3000;

    // for _ in 0 .. ring_bodies {
    //     let mass = rng.random_range(body_min_mass ..= body_max_mass);
    //     let radius = ring_radius + rng.random_range(-ring_delta ..= ring_delta);
    //     let x = rng.random_range(-radius ..= radius);
    //     let y = (if rng.random_bool(0.5) { 1.0 } else { -1.0 }) * f32::sqrt(radius * radius - x * x);
    //     let pos = Vec2::new(x, y);

    //     let vel_mag = f32::sqrt(sun_mass * g / radius);
    //     let vel = Vec2::new(-y, x).normalize() * vel_mag;

    //     new_body(mass, pos, vel);
    // }
}

pub fn setup_scene(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    nbody: NonSendMut<sim::MultiSimulator>,
    scenario: Res<SceneToLoad>,
    mut next_state: ResMut<NextState<SimulationState>>,
) {
    println!("Spawning Scene");

    match *scenario {
        SceneToLoad::Orbit => { orbit_scenario(commands.reborrow(), render_data, nbody); },
    }

    commands.remove_resource::<SceneToLoad>();
    next_state.set(SimulationState::Simulation);

    println!("Spawned Scene");
    // let mut new_body = |mass, posx, posy, velx, vely|
    //     new_body(mass, Vec2::new(posx, posy), Vec2::new(velx, vely), &render_data, &mut commands, &mut nbody);

    // new_body(10.0, -100.0, 0.0, 0.0, 0.0);
    // new_body(20.0, 100.0, 0.0, 0.0, 0.0);
    // new_body(40.0, 0.0, 00.0, 0.0, 0.0);

    // new_body(1.0, -100.0, 25.0, 0.0, 0.0);
    // new_body(2.0, 100.0, 25.0, 0.0, 0.0);
    // new_body(4.0, -100.0, -100.0, 0.0, 0.0);
    // new_body(8.0, 100.0, 100.0, 0.0, 0.0);
    // new_body(16.0, 25.0, 25.0, 0.0, 0.0);

    // for x in -3..3 {
    //     for y in -3..3 {
    //         let mass = 3.0;
    //         let x = x as f32 * 30.0;
    //         let y = y as f32 * 30.0;

    //         new_body(mass, x, y, -y / 50., x / 50.);
    //     }
    // }

    // for x in -50..50 {
    //     for y in -50..50 {
    //         let mass = 1.0;
    //         let x = x as f32 * 10.0;
    //         let y = y as f32 * 10.0;

    //         new_body(mass, x, y, 0.0, 0.0);
    //     }
    // }
}

pub fn destroy_scene(
    mut commands: Commands,
    ui_items: Query<Entity, With<SimEntityTag>>,
) {
    for entity in ui_items {
        commands.entity(entity).despawn();
    }
}
