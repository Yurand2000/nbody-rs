use bevy::prelude::*;
use crate::common::*;
use crate::*;

pub mod scenes;
pub mod update;
pub mod debug;

pub struct SceneRenderPlugin;

impl Plugin for SceneRenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_non_send(sim::MultiSimulator::new_cuda(
                sim::SimulationParams { time_step: 0.1, theta: 0.001, g_constant: 0.667430 }
            ))
            .add_sub_state::<SimUpdateState>()
            .add_sub_state::<SimDebugState>()

            .add_systems(Startup, render_setup)
            .add_systems(OnEnter(SimulationState::Loading), scenes::setup_scene)
            .add_systems(OnExit(SimulationState::Simulation), scenes::destroy_scene)
            .add_systems(FixedUpdate, update::update.run_if(in_state(SimUpdateState::Running)))
            .add_systems(Update, update::process_inputs.run_if(in_state(SimulationState::Simulation)))

            // Simulation Systems
            .add_systems(Update, update::process_inputs_simulate.run_if(in_state(SimUpdateState::Running)))

            // EscMenu Systems

            // Debug Systems
            .add_systems(Update, debug::process_inputs_debug.run_if(in_state(SimUpdateState::Debug)))

            .add_systems(OnEnter(SimDebugState::QuadTree), debug::setup_quadtree)
            .add_systems(Update, debug::process_inputs_debug_quadtree.run_if(in_state(SimDebugState::QuadTree)))
            .add_systems(OnExit(SimDebugState::QuadTree), debug::destroy_quadtree)

            .add_systems(OnEnter(SimDebugState::Forces), debug::setup_forces)
            .add_systems(OnExit(SimDebugState::Forces), debug::destroy_forces)

            .add_systems(OnEnter(SimDebugState::ForceSingle), debug::setup_force_single)
            .add_systems(Update, debug::process_inputs_debug_force_single.run_if(in_state(SimDebugState::ForceSingle)))
            .add_systems(OnExit(SimDebugState::ForceSingle), debug::destroy_force_single)
        ;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(SimulationState = SimulationState::Simulation)]
pub enum SimUpdateState {
    #[default]
    Running,
    Paused,
    Debug,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(SimUpdateState = SimUpdateState::Debug)]
pub enum SimDebugState {
    #[default]
    None,
    QuadTree,
    Forces,
    ForceSingle,
}

#[derive(Component)]
pub struct SimEntityTag;

#[derive(Resource)]
pub struct MyAssetData {
    circle_mesh: Handle<Mesh>,
    square_mesh: Handle<Mesh>,
    segment_mesh: Handle<Mesh>,
    white_material: Handle<ColorMaterial>,
    red_material: Handle<ColorMaterial>,
    yellow_material: Handle<ColorMaterial>,
    force_material: Handle<ColorMaterial>,
}

fn render_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        SimEntityTag,
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
}
