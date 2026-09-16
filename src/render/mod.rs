use bevy::prelude::*;
use crate::common::*;
use crate::render::scenes::SimulationUiTag;
use crate::*;

pub mod scenes;
pub mod update;
pub mod debug;

pub struct SceneRenderPlugin;

impl Plugin for SceneRenderPlugin {
    fn build(&self, app: &mut App) {
        let is_running = in_state(SimMenuState::Running);

        app
            .add_sub_state::<SimMenuState>()
            .add_sub_state::<SimUpdateState>()
            .add_sub_state::<SimPausedState>()

            .add_systems(Startup, (simulator_setup, render_setup))
            .add_systems(OnEnter(SimulationState::Loading), scenes::setup_scene)
            .add_systems(OnExit(SimulationState::Simulation), scenes::destroy_scene)
            .add_systems(OnEnter(SimUpdateState::Running), update_tooltip_text)

            // Simulation Systems
            .add_systems(OnEnter(SimUpdateState::Running), start_simulation)
            .add_systems(Update, (update::process_inputs, update::update_fps_text).run_if( is_running.clone() ))
            .add_systems(FixedUpdate, update::update.run_if( is_running.clone().and_then(in_state(SimUpdateState::Running)) ))
            .add_systems(Update, update::process_inputs_simulate.run_if( is_running.clone().and_then(in_state(SimUpdateState::Running)) ))
            .add_systems(OnExit(SimUpdateState::Running), stop_simulation)

            // EscMenu Systems
            .add_systems(OnEnter(SimMenuState::EscMenu), enter_esc_menu)
            .add_systems(Update, process_inputs_esc_menu.run_if( in_state(SimMenuState::EscMenu)) )
            .add_systems(OnExit(SimMenuState::EscMenu), exit_esc_menu)

            // Debug Systems
            .add_systems(Update, debug::process_inputs_debug.run_if( is_running.clone().and_then(in_state(SimUpdateState::Paused)) ))

            .add_systems(OnEnter(SimPausedState::Paused), update_tooltip_text)

            .add_systems(OnEnter(SimPausedState::DebugQuadTree), (update_tooltip_text, debug::setup_quadtree))
            .add_systems(Update, debug::process_inputs_debug_quadtree.run_if( is_running.clone().and_then(in_state(SimPausedState::DebugQuadTree)) ))
            .add_systems(OnExit(SimPausedState::DebugQuadTree), debug::destroy_quadtree)

            .add_systems(OnEnter(SimPausedState::DebugForces), (update_tooltip_text, debug::setup_forces))
            .add_systems(OnExit(SimPausedState::DebugForces), debug::destroy_forces)

            .add_systems(OnEnter(SimPausedState::DebugForceSingle), (update_tooltip_text, debug::setup_force_single))
            .add_systems(Update, debug::process_inputs_debug_force_single.run_if( is_running.clone().and_then(in_state(SimPausedState::DebugForceSingle)) ))
            .add_systems(OnExit(SimPausedState::DebugForceSingle), debug::destroy_force_single)
        ;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(SimulationState = SimulationState::Simulation)]
pub enum SimMenuState {
    #[default]
    Running,
    EscMenu,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(SimulationState = SimulationState::Simulation)]
pub enum SimUpdateState {
    #[default]
    Running,
    Paused,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(SimUpdateState = SimUpdateState::Paused)]
pub enum SimPausedState {
    #[default]
    Paused,
    DebugQuadTree,
    DebugForces,
    DebugForceSingle,
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

fn simulator_setup(
    mut commands: Commands,
) {
    let (backend, frontend) = sim::async_simulator::new_async_simulator();

    std::thread::spawn(|| {
        smol::block_on(backend.build().run());
    });

    commands.insert_resource(frontend);
}

fn render_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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

const TOOLTIP_RUNNING_TEXT: &str = " WASD ~ Camera Movement\n Mouse Wheel ~ Camera Zoom\n +/- ~ Time Step inc/dec\n Shift +/- ~ Theta param inc/dec\n Space ~ Pause/Debug Simulation\n Esc ~ Menu";
const TOOLTIP_DEBUG_NONE_TEXT: &str = " WASD ~ Camera Movement\n Mouse Wheel ~ Camera Zoom\n Left/Right Arrow ~ Cycle Debug Modes\n Space ~ Resume Simulation\n Esc ~ Menu";
const TOOLTIP_DEBUG_QUADTREE_TEXT: &str = " WASD ~ Camera Movement\n Mouse Wheel ~ Camera Zoom\n Left/Right Arrow ~ Cycle Debug Modes\n Space ~ Resume Simulation\n Up/Down Arrow ~ Quadtree Display level up/down\n Esc ~ Menu";
const TOOLTIP_DEBUG_FORCES_TEXT: &str = " WASD ~ Camera Movement\n Mouse Wheel ~ Camera Zoom\n Left/Right Arrow ~ Cycle Debug Modes\n Space ~ Resume Simulation\n Esc ~ Menu";
const TOOLTIP_DEBUG_FORCE_SINGLE_TEXT: &str = " WASD ~ Camera Movement\n Mouse Wheel ~ Camera Zoom\n Left/Right Arrow ~ Cycle Debug Modes\n Space ~ Resume Simulation\n Up/Down Arrow ~ Select next/previous body\n Esc ~ Menu";

fn update_tooltip_text(
    sim_state: Res<State<SimUpdateState>>,
    debug_state: Option<Res<State<SimPausedState>>>,
    ui_entities: Res<scenes::SimulationUiEntities>,
    mut text_query: Query<&mut Text>,
) {
    let mut update_text = |header: &'static str, text: &'static str| {
        *text_query.get_mut(ui_entities.tooltip_header_text).unwrap() = Text::new(header);
        *text_query.get_mut(ui_entities.tooltip_text).unwrap() = Text::new(text);
    };


    if *sim_state != SimUpdateState::Paused {
        update_text("Controls (Running)", TOOLTIP_RUNNING_TEXT);
        return;
    }

    match *debug_state.unwrap().get() {
        SimPausedState::Paused => { update_text("Controls (Paused)", TOOLTIP_DEBUG_NONE_TEXT); },
        SimPausedState::DebugQuadTree => { update_text("Controls (Debug: QuadTree)", TOOLTIP_DEBUG_QUADTREE_TEXT); },
        SimPausedState::DebugForces => { update_text("Controls (Debug: Forces)", TOOLTIP_DEBUG_FORCES_TEXT); },
        SimPausedState::DebugForceSingle => { update_text("Controls (Debug: Force Single)", TOOLTIP_DEBUG_FORCE_SINGLE_TEXT); },
    }
}

fn process_inputs_esc_menu(
    keys: Res<ButtonInput<bevy::input::keyboard::Key>>,
    ui_entities: Res<scenes::SimulationUiEntities>,
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<SimulationUiTag>)>,
    mut next_menu_state: ResMut<NextState<render::SimMenuState>>,
    mut next_game_state: ResMut<NextState<GameSceneState>>,
    mut next_sim_state: ResMut<NextState<render::SimulationState>>,
    mut nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
) {
    use bevy::input::keyboard::Key::*;
    use crate::menu::*;

    if keys.just_pressed(Escape) {
        next_menu_state.set(SimMenuState::Running);
        return;
    }

    if let Ok((interaction, mut color)) = interaction_query.get_mut(ui_entities.exit_menu_yes_button) {
        match interaction {
            Interaction::Pressed => {
                nbody.reinit_empty();
                next_game_state.set(GameSceneState::MainMenu);
                next_sim_state.set(SimulationState::Unloaded);
            },
            Interaction::Hovered => { *color = BackgroundColor(BUTTON_COLOR_HOVER); },
            Interaction::None => { *color = BackgroundColor(BUTTON_COLOR); },
        }
    }

    if let Ok((interaction, mut color)) = interaction_query.get_mut(ui_entities.exit_menu_no_button) {
        match interaction {
            Interaction::Pressed => { next_menu_state.set(SimMenuState::Running); },
            Interaction::Hovered => { *color = BackgroundColor(BUTTON_COLOR_HOVER); },
            Interaction::None => { *color = BackgroundColor(BUTTON_COLOR); },
        }
    }
}

fn enter_esc_menu(
    ui_entities: Res<scenes::SimulationUiEntities>,
    mut ui_visibility: Query<&mut Visibility>,
) {
    *ui_visibility.get_mut(ui_entities.exit_menu_root).unwrap() = Visibility::Inherited;

}

fn exit_esc_menu(
    ui_entities: Res<scenes::SimulationUiEntities>,
    mut ui_visibility: Query<&mut Visibility>,
) {
    *ui_visibility.get_mut(ui_entities.exit_menu_root).unwrap() = Visibility::Hidden;
}

fn start_simulation(
    mut nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
) {
    nbody.set_run();
}

fn stop_simulation(
    mut nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
) {
    nbody.set_pause();
}
