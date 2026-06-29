use bevy::prelude::*;
use nbody_rs::*;

#[derive(Resource)]
struct MyAssetData {
    circle_mesh: Handle<Mesh>,
    white_material: Handle<ColorMaterial>,
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
        white_material: materials.add(Color::WHITE),
    });

    commands.insert_resource(sim::Simulator::new(
        sim::SimulationParams {
            time_step: 0.1,
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

    // new_body(5.0, -100.0, 0.0, 0.0, 10.0);
    // new_body(5.0, 100.0, 0.0, 0.0, -10.0);
    // new_body(5.0, 0.0, 0.0, 0.0, 0.0);

    for x in -2..3 {
        for y in -0..1 {
            let mass = 3.0;
            let x = x as f32 * 30.0;
            let y = y as f32 * 30.0;

            new_body(mass, x, y, 0.0, 0.0);
        }
    }
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, setup_scene).chain())
        .add_systems(FixedUpdate, update)
        .run();
}