use bevy::prelude::*;
use clap::Parser;
use std::str::FromStr;
use nbody_rs::*;

#[derive(Clone, Copy)]
enum Scenario {
    Grid(usize),
    Orbit,
}

#[derive(Clone, Copy)]
enum Backend {
    CPU,
    CUDA { blocks: u64, threads: u64 },
}

#[derive(Parser, Resource)]
struct Args {
    /// Scenario to simulate: `orbit` or `grid:<bodies>`
    #[arg(short= 's', long= "scenario", value_name= "SCENARIO")]
    scenario: Scenario,

    /// Number of iterations to run
    #[arg(short= 'n', value_name= "ITERATIONS")]
    num_iterations: usize,

    /// Integration step size
    #[arg(short= 't', value_name= "DELTA_T")]
    delta_t: f32,

    /// Barnes-Hut opening angle (theta parameter)
    #[arg(short= 'T', value_name= "THETA")]
    theta: f32,

    /// Backend: `cpu` or `cuda:<blocks>x<threads>`
    #[arg(short= 'b', long= "backend", value_name= "BACKEND")]
    backend: Backend,
}

fn main() {
    let args = Args::parse();
    let (backend, frontend) = sim::async_simulator::new_async_simulator();

    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<common::SimulationState>()
        .add_sub_state::<render::SimUpdateState>()
        .insert_resource(args)
        .insert_resource(frontend)
        .insert_non_send(backend.build())
        .add_systems(Startup, (render::render_setup, minimize_window))
        .add_systems(PostStartup, setup_scenario)
        .add_systems(OnEnter(common::SimulationState::Loading), render::scenes::setup_scene)
        .add_systems(Update, simulate.run_if( in_state(render::SimUpdateState::Running) ))
        .add_systems(OnExit(common::SimulationState::Simulation), render::scenes::destroy_scene)
        .run();
}

fn minimize_window(mut windows: Query<&mut Window>) {
    for mut window in &mut windows {
        window.set_minimized(true);
    }
}

fn setup_scenario(
    args: Res<Args>,
    mut commands: Commands,
    mut next_simulation_state: ResMut<NextState<common::SimulationState>>,
    mut nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
) {
    let scene =
        match args.scenario {
            Scenario::Grid(s) => common::SceneToLoad::Grid(s),
            Scenario::Orbit => common::SceneToLoad::Orbit,
        };

    let params =
        sim::SimulationParams {
            time_step: args.delta_t,
            theta: args.theta,
            g_constant: 0.667430
        };

    match args.backend {
        Backend::CPU => nbody.init_up(params),
        Backend::CUDA { blocks: num_blocks, threads: num_threads } => {
            let cuda_params =
                sim::cuda_simulation::CUDASimulatorParams { num_blocks, num_threads };
            nbody.init_cuda(params, cuda_params)
        },
    }

    next_simulation_state.set(common::SimulationState::Loading);
    commands.insert_resource(common::LoadingData { scene });
}

fn simulate(
    args: Res<Args>,
    mut nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
    mut backend: NonSendMut<sim::async_simulator::AsyncSimulatorBackend>,
    mut next_state: ResMut<NextState<render::SimUpdateState>>,
    bodies: Query<(&mut Transform, &common::BodyId)>,
    mut exit: MessageWriter<AppExit>,
) {
    use nbody_rs::sim::Simulator as _;

    // run backend simulation
    smol::block_on(backend.bench_run(args.num_iterations));

    // update renderer
    nbody.pull_updates();
    for (mut transform, id) in bodies {
        let body = nbody.get_body(id.0);

        transform.translation.x = body.position.x;
        transform.translation.y = body.position.y;
    }

    next_state.set(render::SimUpdateState::Paused);
    exit.write(AppExit::Success);
}

impl FromStr for Scenario {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(':') {
            None if s == "orbit" => Ok(Scenario::Orbit),
            Some(("grid", n)) => n.parse().map(Scenario::Grid).map_err(|e| e.to_string()),
            _ => Err("expected `orbit` or `grid:<bodies>`".to_string()),
        }
    }
}

impl FromStr for Backend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || "expected `cpu` or `cuda:<blocks>x<threads>`".to_string();
        match s.split_once(':') {
            None if s == "cpu" => Ok(Backend::CPU),
            Some(("cuda", p)) => {
                let (b, t) = p.split_once('x').ok_or_else(err)?;
                Ok(Backend::CUDA {
                    blocks: b.parse().map_err(|_| err())?,
                    threads: t.parse().map_err(|_| err())?,
                })
            }
            _ => Err(err()),
        }
    }
}
