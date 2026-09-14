use super::*;

#[derive(Component)]
pub struct NewSceneMenuTag;

#[derive(Component)]
pub enum NewSceneButtonType {
    Start,
    Back,
    CUDAToggle,
}

pub fn create_new_scene_menu(
    mut commands: Commands,
    ui_data: Res<UIData>,
) {
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
                        make_toggle_button(Text::new("CUDA Simulation")),
                        make_radio_button((Text::new("1 CUDA Block"), TextLayout::justify(Justify::Center), TextFont { font_size: FontSize::Px(14.0), ..default() })),
                        make_radio_button((Text::new("2 CUDA Blocks"), TextFont { font_size: FontSize::Px(14.0), ..default() })),
                        make_radio_button((Text::new("4 CUDA Blocks"), TextFont { font_size: FontSize::Px(14.0), ..default() })),
                        make_radio_button((Text::new("8 CUDA Blocks"), TextFont { font_size: FontSize::Px(14.0), ..default() })),
                        make_radio_button((Text::new("16 CUDA Blocks"), TextFont { font_size: FontSize::Px(14.0), ..default() })),
                        make_radio_button((Text::new("1 CUDA Thread"), TextFont { font_size: FontSize::Px(14.0), ..default() })),
                        make_radio_button((Text::new("4 CUDA Threads"), TextFont { font_size: FontSize::Px(14.0), ..default() })),
                        make_radio_button((Text::new("16 CUDA Threads"), TextFont { font_size: FontSize::Px(14.0), ..default() })),
                        make_radio_button((Text::new("64 CUDA Threads"), TextFont { font_size: FontSize::Px(14.0), ..default() })),
                        make_radio_button((Text::new("256 CUDA Threads"), TextFont { font_size: FontSize::Px(14.0), ..default() })),
                        make_radio_button((Text::new("1024 CUDA Threads"), TextFont { font_size: FontSize::Px(14.0), ..default() })),
                        make_flex_grow(1.0),
                        (make_button(Text::new("Start Simulation")), NewSceneButtonType::Start),
                        (make_button(Text::new("Back to Main Menu")), NewSceneButtonType::Back),
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
                                (make_list_button(Text::new("Orbit")), SceneToLoad::Orbit),
                                make_list_button(Text::new("Scenario 1")),
                                make_list_button(Text::new("Scenario 2")),
                            ],
                        ),
                    ],
                ),
            ],
        ));
    });
}

pub fn update_new_scene_menu(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &NewSceneButtonType,
        ),
        Changed<Interaction>
    >,
    mut next_game_state: ResMut<NextState<GameSceneState>>,
    mut next_simulation_state: ResMut<NextState<SimulationState>>,
    mut next_menu_state: ResMut<NextState<MainMenuState>>,
) {
    for (interaction, mut color, button_type) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                match button_type {
                    NewSceneButtonType::Start => {
                        next_game_state.set(GameSceneState::Game);
                        next_simulation_state.set(SimulationState::Loading);
                        commands.insert_resource(SceneToLoad::Orbit);
                    },
                    NewSceneButtonType::Back => { next_menu_state.set(MainMenuState::MainMenu); },
                    NewSceneButtonType::CUDAToggle => {

                    },
                };
            },
            Interaction::Hovered => {
                *color = BUTTON_COLOR_HOVER.into();
            },
            Interaction::None => {
                *color = BUTTON_COLOR.into();
            }
        }
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
