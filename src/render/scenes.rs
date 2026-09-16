use bevy::prelude::*;
use rand::RngExt as _;
use crate::sim::Simulator as _;
use super::*;

#[derive(Component)]
pub struct SimulationUiTag;

#[derive(Resource)]
pub struct SimulationUiEntities {
    pub time_step_text: Entity,
    pub theta_text: Entity,
    pub steps_s_text: Entity,
    pub fps_text: Entity,
    pub tooltip_header_text: Entity,
    pub tooltip_text: Entity,
    pub exit_menu_root: Entity,
    pub exit_menu_yes_button: Entity,
    pub exit_menu_no_button: Entity,
}

fn setup_scene_ui(
    mut commands: Commands,
) {
    let mut time_step_text = Entity::PLACEHOLDER;
    let mut theta_text = Entity::PLACEHOLDER;
    let mut steps_s_text = Entity::PLACEHOLDER;
    let mut fps_text = Entity::PLACEHOLDER;
    let mut tooltip_text = Entity::PLACEHOLDER;
    let mut tooltip_header_text = Entity::PLACEHOLDER;
    let mut exit_menu_root = Entity::PLACEHOLDER;
    let mut exit_menu_yes_button = Entity::PLACEHOLDER;
    let mut exit_menu_no_button = Entity::PLACEHOLDER;

    commands.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            ..default()
        },
        SimEntityTag,
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: px(250),
                height: auto(),
                left: px(5),
                top: px(5),
                padding: UiRect::all(px(5)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceEvenly,
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.7)),
        )).with_children(|parent| {
            time_step_text =
                parent.spawn((
                    Text::new("Time Step: NaN"),
                    SimulationUiTag,
                )).id();

            theta_text =
                parent.spawn((
                    Text::new("Theta: NaN"),
                    SimulationUiTag,
                )).id();

            steps_s_text =
                parent.spawn((
                    Text::new("Steps/s: NaN"),
                    SimulationUiTag,
                )).id();

            fps_text =
                parent.spawn((
                    Text::new("FPS: NaN"),
                    SimulationUiTag,
                )).id();
        });

        parent.spawn((
            Node {
                min_width: px(300),
                right: px(5),
                top: px(5),
                padding: UiRect::all(px(5)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceEvenly,
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.7)),
        )).with_children(|parent| {
            tooltip_header_text =
                parent.spawn((
                    Node {
                        width: percent(100),
                        ..default()
                    },
                    Text::new("Controls"),
                    TextLayout::justify(Justify::Center),
                    TextFont { font_size: FontSize::Px(18.0), ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    SimulationUiTag,
                )).id();

            tooltip_text =
                parent.spawn((
                    Text::new(""),
                    TextLayout::justify(Justify::Right),
                    TextFont { font_size: FontSize::Px(14.0), ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    SimulationUiTag,
                )).id();
        });

        let mut exit_menu_root_cmds =
            parent.spawn((
                Node {
                    min_width: px(200),
                    min_height: px(50),
                    margin: UiRect::all(auto()),
                    flex_direction: FlexDirection::Column,

                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.7)),
                Visibility::Hidden,
                SimulationUiTag,
            ));

        exit_menu_root_cmds.with_children(|parent| {
            parent.spawn((
                Node {
                    margin: UiRect::all(px(10)),
                    ..default()
                },
                children![(
                    Text::new("Are you sure you\nwant to quit?"),
                    TextLayout::justify(Justify::Center),
                )]
            ));

            parent.spawn((
                Node {
                    margin: UiRect::all(px(10)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    ..default()
                },
            )).with_children(|parent| {
                exit_menu_yes_button =
                    parent.spawn((
                        Node {
                            width: px(50),
                            ..default()
                        },
                        Button,
                        Text::new("Yes"),
                        TextLayout::justify(Justify::Center),
                        BackgroundColor(Color::srgba(0.4, 0.4, 0.4, 0.7)),
                        SimulationUiTag,
                    )).id();

                exit_menu_no_button =
                    parent.spawn((
                        Node {
                            width: px(50),
                            ..default()
                        },
                        Button,
                        Text::new("No"),
                        TextLayout::justify(Justify::Center),
                        BackgroundColor(Color::srgba(0.4, 0.4, 0.4, 0.7)),
                        SimulationUiTag,
                    )).id();
            });
        });

        exit_menu_root = exit_menu_root_cmds.id();
    });

    commands.insert_resource(SimulationUiEntities {
        time_step_text,
        theta_text,
        steps_s_text,
        fps_text,
        tooltip_text,
        tooltip_header_text,
        exit_menu_root,
        exit_menu_yes_button,
        exit_menu_no_button,
    });
}

fn new_body(
    mass: f32,
    pos: Vec2,
    vel: Vec2,
    render_data: &MyAssetData,
    commands: &mut Commands,
    nbody: &mut sim::async_simulator::AsyncSimulatorFrontend,
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
    mut nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
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

fn two_body_orbit_scenario(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    mut nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
) {
    let mut new_body = |mass, pos, vel|
        new_body(mass, pos, vel, &render_data, &mut commands, &mut nbody);

    new_body(10000.0, Vec2::new(-100.0, 0.0), Vec2::ZERO);
    new_body(100.0, Vec2::new(100.0, 0.0), Vec2::new(0.0, 3.0));
}

fn grid_scenario(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    mut nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
) {
    let mut new_body = |mass, posx, posy, velx, vely|
        new_body(mass, Vec2::new(posx, posy), Vec2::new(velx, vely), &render_data, &mut commands, &mut nbody);

    // new_body(10.0, -100.0, 0.0, 0.0, 0.0);
    // new_body(20.0, 100.0, 0.0, 0.0, 0.0);
    // new_body(40.0, 0.0, 00.0, 0.0, 0.0);
    //
    // new_body(1.0, -100.0, 25.0, 0.0, 0.0);
    // new_body(2.0, 100.0, 25.0, 0.0, 0.0);
    // new_body(4.0, -100.0, -100.0, 0.0, 0.0);
    // new_body(8.0, 100.0, 100.0, 0.0, 0.0);
    // new_body(16.0, 25.0, 25.0, 0.0, 0.0);
    //
    // for x in -3..3 {
    //     for y in -3..3 {
    //         let mass = 3.0;
    //         let x = x as f32 * 30.0;
    //         let y = y as f32 * 30.0;
    //
    //         new_body(mass, x, y, -y / 50., x / 50.);
    //     }
    // }

    for x in -30..30 {
        for y in -30..30 {
            let mass = 1.0;
            let x = x as f32 * 10.0;
            let y = y as f32 * 10.0;

            new_body(mass, x, y, 0.0, 0.0);
        }
    }
}

pub fn setup_scene(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
    load_data: Res<LoadingData>,
    mut next_state: ResMut<NextState<SimulationState>>,
) {
    println!("Spawning Scene");

    commands.spawn((
        Camera2d,
        SimEntityTag,
        Transform::from_scale([1.0, 1.0, 1.0].into())
    ));

    setup_scene_ui(commands.reborrow());

    match load_data.scene {
        SceneToLoad::TwoBodies => { two_body_orbit_scenario(commands.reborrow(), render_data, nbody); },
        SceneToLoad::Grid => { grid_scenario(commands.reborrow(), render_data, nbody); },
        SceneToLoad::Orbit => { orbit_scenario(commands.reborrow(), render_data, nbody); },
    }

    commands.remove_resource::<LoadingData>();
    next_state.set(SimulationState::Simulation);

    println!("Spawned Scene");
}

pub fn destroy_scene(
    mut commands: Commands,
    mut nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
    ui_items: Query<Entity, With<SimEntityTag>>,
) {
    for entity in ui_items {
        commands.entity(entity).despawn();
    }

    nbody.reset();
}
