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
            time_step: 1.0,
            typ: sim::SimulationType::SingleCore,
            theta: 1.0,
            grav_constant: 6.67430,
        }));
}

fn setup_scene(
    mut commands: Commands,
    render_data: Res<MyAssetData>,
    mut nbody: ResMut<sim::Simulator>,
) {
    let circle = &render_data.circle_mesh;
    let white = &render_data.white_material;

    for x in -10..10 {
        for y in -10..10 {
            let mass = 3.0;
            let x = x as f32 * 30.0;
            let y = y as f32 * 30.0;

            let body = sim::Body {
                mass,
                position: [x, y].into(),
                velocity: [0.0, 0.0].into(),
            };

            let id = BodyId(nbody.insert_body(body));

            commands.spawn((
                id,
                Mesh2d(circle.to_owned()),
                MeshMaterial2d(white.to_owned()),
                Transform::from_xyz(x, y, 0.0)
                    .with_scale(Vec3::from([mass, mass, 1.0]))
            ));
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, setup_scene).chain())
        .run();
}