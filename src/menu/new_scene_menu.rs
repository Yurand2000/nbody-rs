use crate::sim;

use super::*;

#[derive(Component)]
pub struct NewSceneMenuTag;

#[derive(Component)]
pub struct NewSceneStartButton;

#[derive(Component)]
pub struct NewSceneBackButton;

#[derive(Component)]
pub struct NewSceneCUDAToggleButton;

#[derive(Component)]
pub struct NewSceneCUDAParamsTag;

#[derive(Component)]
pub struct NewSceneCUDABlocksInput;

#[derive(Component)]
pub struct NewSceneCUDAThreadsInput;

pub fn create_new_scene_menu(
    mut commands: Commands,
    ui_data: Res<UIData>,
) {
    #[cfg(feature = "cuda")]
    let cuda_button =
        (make_toggle_button(Text::new("CUDA Simulation")), NewSceneCUDAToggleButton);

    #[cfg(not(feature = "cuda"))]
    let cuda_button =
        (make_toggle_button((Text::new("CUDA Simulation"), TextColor(TEXT_COLOR_LIGHT))), NewSceneCUDAToggleButton, DisabledButton);

    commands.entity(ui_data.main_ui_entity).with_children(|parent| {
        parent.spawn((
            Node {
                min_height: px(500),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            NewSceneMenuTag,
            BackgroundColor(WINDOW_COLOR),
            children![
                (
                    Node {
                        min_width: px(300),
                        height: percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::FlexStart,
                        border: UiRect::all(px(5)),
                        padding: UiRect::all(px(5)),
                        ..default()
                    },
                    children![
                        make_text(
                            Text::new("New Scene Settings"),
                            Some(FontSize::Px(24.0)),
                            None,
                        ),
                        cuda_button,
                        (
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Start,
                                justify_content: JustifyContent::FlexStart,
                                ..default()
                            },
                            Visibility::Hidden,
                            NewSceneCUDAParamsTag,
                            children![
                                make_text_input("16", Text::new("CUDA Blocks"), NewSceneCUDABlocksInput),
                                make_text_input("256", Text::new("CUDA Threads per Block"), NewSceneCUDAThreadsInput),
                            ]
                        ),
                        make_flex_grow(1.0),
                        (make_button(Text::new("Start Simulation")), NewSceneStartButton),
                        (make_button(Text::new("Back to Main Menu")), NewSceneBackButton),
                    ],
                ),
                (
                    Node {
                        min_width: px(300),
                        height: percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        border: UiRect::all(px(5)),
                        padding: UiRect::all(px(5)),
                        ..default()
                    },
                    children![
                        make_text(
                            Text::new("Available Scenes"),
                            Some(FontSize::Px(24.0)),
                            None,
                        ),
                        (
                            Node {
                                width: percent(100.0),
                                height: auto(),
                                padding: UiRect::all(px(5)),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::FlexStart,
                                justify_content: JustifyContent::FlexStart,
                                flex_grow: 1.0,
                                ..default()
                            },
                            BackgroundColor(WINDOW_COLOR_LIGHT),
                            children![
                                (make_list_button(Text::new("Two Bodies")), SceneToLoad::TwoBodies),
                                (make_list_button(Text::new("Grid")), SceneToLoad::Grid),
                                (make_list_button(Text::new("Orbit")), SceneToLoad::Orbit),
                            ],
                        ),
                    ],
                ),
            ],
        ));
    });
}

pub fn on_cuda_toggle_button(
    interaction_query: Single<&Interaction, (Changed<Interaction>, With<NewSceneCUDAToggleButton>, Without<DisabledButton>)>,
    mut cuda_params_query: Query<&mut Visibility, With<NewSceneCUDAParamsTag>>,
) {
    match interaction_query.into_inner() {
        Interaction::Pressed => {
            cuda_params_query.single_mut().unwrap().toggle_inherited_hidden();
        },
        _ => (),
    }
}

pub fn on_start_button(
    mut commands: Commands,
    use_cuda_query: Query<Has<bevy::ui::Checked>, With<NewSceneCUDAToggleButton>>,
    #[allow(unused_variables)]
    cuda_blocks_param_query: Query<&bevy::text::EditableText, With<NewSceneCUDABlocksInput>>,
    #[allow(unused_variables)]
    cuda_threads_param_query: Query<&bevy::text::EditableText, With<NewSceneCUDAThreadsInput>>,
    interaction_query: Single<&Interaction, (Changed<Interaction>, With<NewSceneStartButton>)>,
    selected_scene_query: Query<(Has<bevy::ui::Checked>, &SceneToLoad)>,
    mut next_game_state: ResMut<NextState<GameSceneState>>,
    mut next_simulation_state: ResMut<NextState<SimulationState>>,
    mut nbody: NonSendMut<sim::MultiSimulator>,
) {
    match interaction_query.into_inner() {
        Interaction::Pressed => {
            let mut selected_scene = None;
            for (checked, scene) in selected_scene_query {
                if !checked { continue; }

                selected_scene = Some(*scene);
            }

            let Some(scene) = selected_scene else { return; };

            let default_params =
                sim::SimulationParams { time_step: 0.1, theta: 0.1, g_constant: 0.667430 };

            let use_cuda = use_cuda_query.single().unwrap();
            if use_cuda {
                #[cfg(feature = "cuda")]
                {
                    let Ok(num_blocks) = cuda_blocks_param_query.single().unwrap().value().to_string().parse() else { return; };
                    let Ok(num_threads) = cuda_threads_param_query.single().unwrap().value().to_string().parse() else { return; };

                    let cuda_params =
                        sim::cuda_simulation::CUDASimulatorParams { num_blocks, num_threads };

                    nbody.init_cuda(default_params, cuda_params);
                }
            } else {
                nbody.init_up(default_params);
            }

            next_game_state.set(GameSceneState::Game);
            next_simulation_state.set(SimulationState::Loading);
            commands.insert_resource(LoadingData { scene });
        },
        _ => (),
    }
}

pub fn on_back_button(
    interaction_query: Single<&Interaction, (Changed<Interaction>, With<NewSceneBackButton>)>,
    mut next_state: ResMut<NextState<MainMenuState>>,
) {
    match interaction_query.into_inner() {
        Interaction::Pressed => { next_state.set(MainMenuState::MainMenu); },
        _ => (),
    }
}

pub fn destroy_new_scene_menu(
    mut commands: Commands,
    ui_items: Query<Entity, With<NewSceneMenuTag>>,
) {
    for entity in ui_items {
        commands.entity(entity).despawn();
    }
}
