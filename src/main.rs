use std::f32;
use bevy::{input::mouse::MouseWheel, prelude::*};
use nbody_rs::*;
use nbody_rs::sim::{Simulator as _, SimulatorExt as _};
use rand::RngExt;

#[derive(Resource)]
struct MyAssetData {
    circle_mesh: Handle<Mesh>,
    square_mesh: Handle<Mesh>,
    segment_mesh: Handle<Mesh>,
    white_material: Handle<ColorMaterial>,
    red_material: Handle<ColorMaterial>,
    yellow_material: Handle<ColorMaterial>,
    force_material: Handle<ColorMaterial>,
}

#[derive(Component)]
struct BodyId(sim::BodyId);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Transform::from_scale([1.0, 1.0, 1.0].into())
    ));

    commands.insert_resource(MyAssetData {
        circle_mesh: meshes.add(Circle::new(1.0)),
        square_mesh: meshes.add(Rectangle::new(1.0, 1.0)),
        segment_mesh: meshes.add(Segment2d::new(Vec2::ZERO, Vec2::Y)),
        white_material: materials.add(Color::WHITE),
        red_material: materials.add(Color::linear_rgba(1.0, 0.0, 0.0, 0.1)),
        yellow_material: materials.add(Color::linear_rgba(1.0, 1.0, 0.0, 0.1)),
        force_material: materials.add(Color::linear_rgba(1.0, 0.2, 1.0, 1.0)),
    });

    commands.insert_resource(Time::<Fixed>::from_hz(1.0));
}

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

            let theta = rng.random::<f32>() * f32::consts::TAU;
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

fn setup_scene(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    mut nbody: NonSendMut<sim::MultiSimulator>,
) {
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

    println!("Spawning Scene");
    orbit_scenario(commands, render_data, nbody);
    println!("Spawned Scene");
}

fn update(
    mut nbody: NonSendMut<sim::MultiSimulator>,
    bodies: Query<(&mut Transform, &BodyId)>,
) {
    let start = std::time::Instant::now();

    nbody.step();

    let total = std::time::Instant::now() - start;
    println!("step duration: {total:?}");

    let start = std::time::Instant::now();

    nbody.pull_updates();

    let total = std::time::Instant::now() - start;
    println!("pull duration: {total:?}");

    for (mut transform, id) in bodies {
        let body = nbody.get_body(id.0);

        transform.translation.x = body.position.x;
        transform.translation.y = body.position.y;
    }
}

fn process_inputs(
    keys: Res<ButtonInput<bevy::input::keyboard::Key>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    state: Res<State<render::SimState>>,
    mut next_state: ResMut<NextState<render::SimState>>,
    camera: Single<(&mut Transform, &Camera2d)>,
) {
    use render::SimState::*;
    use bevy::input::keyboard::Key::*;

    if keys.just_pressed(Space) {
        match *state.get() {
            Simulate => next_state.set(Debug),
            Debug => next_state.set(Simulate),
        }
    }

    let (mut camera, _) = camera.into_inner();

    let mut mov = Vec3::ZERO;
    if keys.pressed(Character("w".into())) {
        mov.y += 1.0;
    }

    if keys.pressed(Character("a".into())) {
        mov.x -= 1.0;
    }

    if keys.pressed(Character("s".into())) {
        mov.y -= 1.0;
    }

    if keys.pressed(Character("d".into())) {
        mov.x += 1.0;
    }

    camera.translation += mov * 10.0;

    for evt in mouse_wheel.read() {
        if evt.y > 0.0 {
            camera.scale.x /= 1.2;
            camera.scale.y /= 1.2;
        } else {
            camera.scale.x *= 1.2;
            camera.scale.y *= 1.2;
        }
    }
}

fn process_inputs_simulate(
    keys: Res<ButtonInput<bevy::input::keyboard::Key>>,
    mut nbody: NonSendMut<sim::MultiSimulator>,
) {
    let mut params = nbody.get_params();

    if keys.pressed(bevy::input::keyboard::Key::Character("+".into())) &&
       !keys.pressed(bevy::input::keyboard::Key::Shift)
    {
        params.time_step *= 1.05;
    }

    if keys.pressed(bevy::input::keyboard::Key::Character("-".into())) &&
       !keys.pressed(bevy::input::keyboard::Key::Shift)
    {
        params.time_step /= 1.05;
    }

    if keys.just_pressed(bevy::input::keyboard::Key::Character("+".into())) &&
       keys.pressed(bevy::input::keyboard::Key::Shift)
    {
        params.theta *= 2.0;
    }

    if keys.just_pressed(bevy::input::keyboard::Key::Character("-".into())) &&
       keys.pressed(bevy::input::keyboard::Key::Shift)
    {
        params.theta /= 2.0;
    }

    nbody.set_params(params);
}

fn process_inputs_debug(
    keys: Res<ButtonInput<bevy::input::keyboard::Key>>,
    state: Res<State<render::DebugState>>,
    mut next_state: ResMut<NextState<render::DebugState>>,
) {
    use render::DebugState::*;

    if keys.just_pressed(bevy::input::keyboard::Key::ArrowLeft) {
        match *state.get() {
            QuadTree => next_state.set(Forces),
            Forces => next_state.set(ForceSingle),
            ForceSingle => next_state.set(QuadTree),
        }
    }

    if keys.just_pressed(bevy::input::keyboard::Key::ArrowRight) {
        match *state.get() {
            QuadTree => next_state.set(ForceSingle),
            Forces => next_state.set(QuadTree),
            ForceSingle => next_state.set(Forces),
        }
    }
}

fn process_inputs_debug_quadtree(
    keys: Res<ButtonInput<bevy::input::keyboard::Key>>,
    mut debug_data: ResMut<QuadtreeDebugState>,
    quadtree_items: Query<(&mut Visibility, &QuadTreeItem)>,
) {
    if keys.just_pressed(bevy::input::keyboard::Key::ArrowUp) {
        if debug_data.view_level < debug_data.max_level {
            debug_data.view_level += 1;
        }
    }

    if keys.just_pressed(bevy::input::keyboard::Key::ArrowDown) {
        if debug_data.view_level > 0 {
            debug_data.view_level -= 1;
        }
    }

    for (mut vis, item) in quadtree_items {
        if item.level == debug_data.view_level {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn process_inputs_debug_force_single(
    keys: Res<ButtonInput<bevy::input::keyboard::Key>>,
    mut debug_data: ResMut<ForceSingleDebugState>,
    nbody: NonSend<sim::MultiSimulator>,
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

#[derive(Resource)]
pub struct QuadtreeDebugState {
    view_level: usize,
    max_level: usize,
}

#[derive(Component)]
pub struct QuadTreeItem {
    level: usize
}

fn draw_quadtree(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    nbody: NonSend<sim::MultiSimulator>,
) {
    use sim::QuadTreeUtils;

    let Some(tree) = nbody.get_quadtree() else { return; };
    let max_vis_depth = tree.depth.min(8);

    commands.insert_resource(QuadtreeDebugState {
        view_level: 0,
        max_level: max_vis_depth
    });

    let circle = &render_data.circle_mesh;
    let mat = &render_data.force_material;
    let square = &render_data.square_mesh;
    let red = &render_data.red_material;
    let yellow = &render_data.yellow_material;
    for depth in 0 ..= max_vis_depth {
        let cells = tree.layer_at_depth(depth);

        let cell_size = QuadTreeUtils::layer_width(depth);
        for (idx, cell) in cells.iter().enumerate() {
            let cell_pos =
                tree.norm.invert_normalize(
                    QuadTreeUtils::position_at_layer(depth, idx)
                    + sim::Vec2f::new(QuadTreeUtils::layer_width(depth + 1), QuadTreeUtils::layer_width(depth + 1))
                );

            let cell_size = tree.norm.invert_normalize_size(
                sim::Vec2f::new(cell_size, cell_size)
            );

            let text_size = (2.0 / (depth + 1) as f32).min(1.0);

            commands.spawn((
                Visibility::Visible,
                Transform::from_xyz(cell_pos.x, cell_pos.y, 1.0),
                QuadTreeItem { level: depth },
            ))
            .with_child((
                Mesh2d(circle.to_owned()),
                MeshMaterial2d(mat.to_owned()),
                Transform::from_xyz(cell.position.x - cell_pos.x, cell.position.y - cell_pos.y, 1.0)
                    .with_scale([cell.mass.sqrt(), cell.mass.sqrt(), 1.0].into()),
            ))
            .with_child((
                Mesh2d(square.to_owned()),
                if cell.mass == 0.0 {
                    MeshMaterial2d(yellow.to_owned())
                } else {
                    MeshMaterial2d(red.to_owned())
                },
                Transform::from_scale(Vec3::from([cell_size.x - 1.0, cell_size.y - 1.0, 1.0])),
            ))
            .with_child((
                Text2d::new(format!("{}", idx)),
                Transform::from_scale(Vec3::from([text_size, text_size, 1.0]))
            ));
        }
    }
}

fn destroy_quadtree(
    mut commands: Commands,
    quadtree_items: Query<Entity, With<QuadTreeItem>>
) {
    commands.remove_resource::<QuadtreeDebugState>();

    for entity in quadtree_items {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
pub struct ForceTag;

fn draw_forces(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    mut nbody: NonSendMut<sim::MultiSimulator>,
) {
    let Some(tree) = nbody.get_quadtree() else { return; };
    let Some(forces) = nbody.get_forces(&tree) else { return; };

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
        ));
    }
}

fn destroy_forces(
    mut commands: Commands,
    quadtree_items: Query<Entity, With<ForceTag>>
) {
    for entity in quadtree_items {
        commands.entity(entity).despawn();
    }
}

#[derive(Resource)]
pub struct ForceSingleDebugState {
    curr_body: usize,
}

#[derive(Component)]
pub struct ForceSingleTag { body_id: usize }

fn draw_force_single(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    mut nbody: NonSendMut<sim::MultiSimulator>,
) {
    commands.insert_resource(ForceSingleDebugState {
        curr_body: 0,
    });

    let Some(tree) = nbody.get_quadtree() else { return; };

    let circle = &render_data.circle_mesh;
    let segment = &render_data.segment_mesh;
    let red = &render_data.red_material;
    let mat = &render_data.force_material;

    nbody.pull_updates();
    for (body_id, body, forces) in nbody.get_bodies().iter().enumerate()
        .map(|(body_id, body)| (body_id, body, nbody.get_body_forces(&tree, body_id)))
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
        ));

        for sim::SingleBodyForce { com, force } in forces.into_iter() {
            let magnitude = (force.magnitude() - min_force) / diff_force;

            commands.spawn((
                Visibility::Visible,
                Mesh2d(segment.to_owned()),
                MeshMaterial2d(mat.to_owned()),
                Transform::from_translation([com.x, com.y, 0.0].into()),
                ForceSingleTag { body_id },
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

fn destroy_force_single(
    mut commands: Commands,
    quadtree_items: Query<Entity, With<ForceSingleTag>>,
) {
    for entity in quadtree_items {
        commands.entity(entity).despawn();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_non_send_resource(sim::MultiSimulator::new_cuda(
            sim::SimulationParams { time_step: 0.1, theta: 1.0, g_constant: 0.667430 }
        ))
        .init_state::<render::SimState>()
        .add_sub_state::<render::DebugState>()
        .add_systems(Startup, (setup, setup_scene).chain())
        .add_systems(Update, process_inputs)
        .add_systems(Update, process_inputs_simulate.run_if(in_state(render::SimState::Simulate)))
        .add_systems(Update, process_inputs_debug.run_if(in_state(render::SimState::Debug)))
        .add_systems(Update, process_inputs_debug_quadtree.run_if(in_state(render::DebugState::QuadTree)))
        .add_systems(Update, process_inputs_debug_force_single.run_if(in_state(render::DebugState::ForceSingle)))
        .add_systems(FixedUpdate, update.run_if(in_state(render::SimState::Simulate)))
        .add_systems(OnEnter(render::DebugState::QuadTree), draw_quadtree)
        .add_systems(OnExit(render::DebugState::QuadTree), destroy_quadtree)
        .add_systems(OnEnter(render::DebugState::Forces), draw_forces)
        .add_systems(OnExit(render::DebugState::Forces), destroy_forces)
        .add_systems(OnEnter(render::DebugState::ForceSingle), draw_force_single)
        .add_systems(OnExit(render::DebugState::ForceSingle), destroy_force_single)
        .run();
}