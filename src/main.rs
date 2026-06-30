use bevy::prelude::*;
use nbody_rs::*;

#[derive(Resource)]
struct MyAssetData {
    circle_mesh: Handle<Mesh>,
    square_mesh: Handle<Mesh>,
    white_material: Handle<ColorMaterial>,
    red_material: Handle<ColorMaterial>,
    yellow_material: Handle<ColorMaterial>,
}

#[derive(Component)]
struct BodyId(sim::BodyId);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.insert_resource(MyAssetData {
        circle_mesh: meshes.add(Circle::new(1.0)),
        square_mesh: meshes.add(Rectangle::new(1.0, 1.0)),
        white_material: materials.add(Color::WHITE),
        red_material: materials.add(Color::linear_rgba(1.0, 0.0, 0.0, 0.1)),
        yellow_material: materials.add(Color::linear_rgba(1.0, 1.0, 0.0, 0.1)),
    });

    commands.insert_resource(sim::Simulator::new(
        sim::SimulationParams {
            time_step: 0.01,
            typ: sim::SimulationType::SingleCore,
            theta: 0.0,
            grav_constant: 6.67430,
        }));

    commands.insert_resource(Time::<Fixed>::from_hz(30.0));
}

fn setup_scene(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    mut nbody: ResMut<sim::Simulator>,
) {
    let circle = &render_data.circle_mesh;
    let white = &render_data.white_material;

    let mut new_body = |mass, pos_x, pos_y, vel_x, vel_y| {
        let body = sim::Body {
            mass,
            position: sim::Vec2::new(pos_x, pos_y),
            velocity: sim::Vec2::new(vel_x, vel_y),
        };

        let id = BodyId(nbody.insert_body(body));

        commands.spawn((
            id,
            Mesh2d(circle.to_owned()),
            MeshMaterial2d(white.to_owned()),
            Transform::from_xyz(pos_x, pos_y, 0.0)
                .with_scale(Vec3::from([mass, mass, 1.0]))
        ));
    };

    new_body(5.0, -100.0, 25.0, 0.0, 10.0);
    new_body(5.0, 100.0, 25.0, 0.0, -10.0);
    new_body(5.0, -100.0, -100.0, 0.0, 10.0);
    new_body(5.0, 100.0, 100.0, 0.0, -10.0);
    new_body(5.0, 25.0, 25.0, 0.0, 0.0);

    // for x in -3..3 {
    //     for y in -3..3 {
    //         let mass = 3.0;
    //         let x = x as f32 * 30.0;
    //         let y = y as f32 * 30.0;

    //         new_body(mass, x, y, 0.0, 0.0);
    //     }
    // }
}

fn update(
    mut nbody: ResMut<sim::Simulator>,
    bodies: Query<(&mut Transform, &BodyId)>,
) {
    nbody.step();

    for (mut transform, id) in bodies {
        let body = nbody.get_body(id.0);

        transform.translation.x = body.position.x;
        transform.translation.y = body.position.y;
    }
}

fn process_inputs(
    keys: Res<ButtonInput<bevy::input::keyboard::Key>>,
    state: Res<State<render::SimState>>,
    mut next_state: ResMut<NextState<render::SimState>>,
    debug_state: Option<ResMut<DebugState>>,
    quadtree_items: Query<(&mut Visibility, &QuadTreeItem)>,
) {
    if keys.just_pressed(bevy::input::keyboard::Key::Space) {
        match *state.get() {
            render::SimState::Simulate =>
                next_state.set(render::SimState::Debug),
            render::SimState::Debug =>
                next_state.set(render::SimState::Simulate),
        }
    }

    let Some(mut debug_state) = debug_state else { return; };

    if keys.just_pressed(bevy::input::keyboard::Key::ArrowUp) {
        if debug_state.view_level < debug_state.max_level {
            debug_state.view_level += 1;
        }
    }

    if keys.just_pressed(bevy::input::keyboard::Key::ArrowDown) {
        if debug_state.view_level > 0 {
            debug_state.view_level -= 1;
        }
    }

    for (mut vis, item) in quadtree_items {
        if item.level == debug_state.view_level {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

#[derive(Resource)]
pub struct DebugState {
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
    nbody: Res<sim::Simulator>,
) {
    let (size_params, tree) = nbody.get_quadtree();

    commands.insert_resource(DebugState {
        view_level: 0,
        max_level: tree.depth
    });

    let square = &render_data.square_mesh;
    let red = &render_data.red_material;
    let yellow = &render_data.yellow_material;
    for depth in 0 ..= tree.depth {
        let cells = tree.layer_at_depth(depth);

        let cell_size = sim::up_simulation::QuadTreeUtils::layer_width(depth);
        for (idx, cell) in cells.iter().enumerate() {
            let cell_pos =
                size_params.invert_normalize(
                    sim::up_simulation::QuadTreeUtils
                        ::position_at_layer(depth, idx)
                );

            let cell_size = size_params.invert_normalize_size(
                sim::Vec2::new(cell_size, cell_size)
            );

            commands.spawn((
                Visibility::Visible,
                Mesh2d(square.to_owned()),
                if cell.mass == 0.0 {
                    MeshMaterial2d(yellow.to_owned())
                } else {
                    MeshMaterial2d(red.to_owned())
                },
                Transform::from_xyz(cell_pos.x, cell_pos.y, 0.0)
                    .with_scale(Vec3::from([cell_size.x - 1.0, cell_size.y - 1.0, 1.0])),
                QuadTreeItem { level: depth },
            ));
        }
    }
}

fn destroy_quadtree(
    mut commands: Commands,
    quadtree_items: Query<Entity, With<QuadTreeItem>>
) {
    commands.remove_resource::<DebugState>();

    for entity in quadtree_items {
        commands.entity(entity).despawn();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<render::SimState>()
        .add_systems(Startup, (setup, setup_scene).chain())
        .add_systems(Update, process_inputs)
        .add_systems(FixedUpdate, update.run_if(in_state(render::SimState::Simulate)))
        .add_systems(OnEnter(render::SimState::Debug), draw_quadtree)
        .add_systems(OnExit(render::SimState::Debug), destroy_quadtree)
        .run();
}