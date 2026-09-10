use bevy::{camera::visibility::RenderLayers, prelude::*};
use crate::{common::*, render};

pub struct MenuPlugin;

#[derive(Resource)]
pub struct UIData {
    camera_entity: Entity,
    main_ui_entity: Entity,
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_sub_state::<MainMenuState>()
            .add_systems(Startup, setup_ui)
            .add_systems(OnEnter(MainMenuState::MainMenu), create_main_menu)
            .add_systems(Update, update_main_menu.run_if(in_state(MainMenuState::MainMenu)))
            .add_systems(OnExit(MainMenuState::MainMenu), destroy_main_menu)
            .add_systems(OnEnter(MainMenuState::About), create_about_menu)
            .add_systems(Update, update_about_menu.run_if(in_state(MainMenuState::About)))
            .add_systems(OnExit(MainMenuState::About), destroy_about_menu)
            .add_systems(OnEnter(MainMenuState::NewScene), create_new_scene_menu)
            .add_systems(Update, update_new_scene_menu.run_if(in_state(MainMenuState::NewScene)))
            .add_systems(OnExit(MainMenuState::NewScene), destroy_new_scene_menu)
        ;
    }
}

fn setup_ui(
    mut commands: Commands,
    mut next_state: ResMut<NextState<MainMenuState>>,
) {
    let camera_entity =
        commands.spawn((
            Camera2d,
            Camera {
                order: 1,
                ..default()
            },
            RenderLayers::layer(1),
        )).id();

    let main_ui_entity =
        commands.spawn((
            UiTargetCamera(camera_entity),
            Node {
                width: percent(100.0),
                height: percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            RenderLayers::layer(1),
        )).id();

    commands.insert_resource(
        UIData { camera_entity, main_ui_entity }
    );

    next_state.set(MainMenuState::MainMenu);
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(GameSceneState = GameSceneState::MainMenu)]
pub enum MainMenuState {
    #[default]
    Loading,
    MainMenu,
    NewScene,
    About,
}

const WINDOW_COLOR: Color = Color::srgba(0.2, 0.2, 0.2, 0.7);
const WINDOW_COLOR_LIGHT: Color = Color::srgba(0.25, 0.25, 0.25, 0.7);
const BORDER_COLOR: Color = Color::BLACK;
const BUTTON_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
const BUTTON_COLOR_HOVER: Color = Color::srgb(0.5, 0.5, 0.5);
const TEXT_COLOR: Color = Color::WHITE;
const TEXT_COLOR_LIGHT: Color = Color::srgb(0.4, 0.4, 0.4);
const TEXT_COLOR_LIGHT_HOVER: Color = Color::srgb(0.6, 0.6, 0.6);
const CHECKBOX_TRUE: Color = Color::srgb(0.0, 1.0, 0.0);
const CHEKCBOX_FALSE: Color = Color::NONE;

fn make_text(
    text: Text,
    size: Option<FontSize>,
    color: Option<Color>
) -> impl Bundle {
    (
        text,
        TextFont {
            font_size: size.unwrap_or_default(),
            ..default()
        },
        TextColor(color.unwrap_or(TEXT_COLOR)),
    )
}

fn make_button(
    text: Text,
) -> impl Bundle {
    (
        Button,
        Node {
            width: percent(100.0),
            height: px(50),
            border: UiRect::all(px(2)),
            margin: UiRect::vertical(px(2)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(5)),
            ..default()
        },
        BorderColor::all(BORDER_COLOR),
        BackgroundColor(BUTTON_COLOR),
        children![ make_text(text, None, None) ]
    )
}

fn make_checkbox(
    text: Text
) -> impl Bundle {
    (
        bevy::ui_widgets::Checkbox,
        Node {
            border: UiRect::all(px(2)),
            margin: UiRect::vertical(px(2)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            ..default()
        },
        children![
            (
                Node {
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(5)),
                    ..default()
                },
                BorderColor::all(BORDER_COLOR),
                BackgroundColor(BUTTON_COLOR),
                children![(
                    Node {
                        width: px(12),
                        height: px(12),
                        margin: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(5)),
                        ..default()
                    },
                    BackgroundColor(CHECKBOX_TRUE),
                )]
            ),
            (
                Node {
                    border: UiRect::axes(px(10), px(2)),
                    margin: UiRect::vertical(px(2)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![ make_text(
                    text,
                    Some(FontSize::Px(22.0)),
                    None,
                )]
            )
        ]
    )
}

fn make_flex_grow(
    amount: f32
) -> impl Bundle {
    Node {
        flex_grow: amount,
        ..default()
    }
}



#[derive(Component)]
pub struct MainMenuTag;

#[derive(Component)]
pub enum MainMenuButtonType {
    NewScene,
    About,
    Exit
}

fn create_main_menu(
    mut commands: Commands,
    ui_data: Res<UIData>,
) {
    commands.entity(ui_data.main_ui_entity).with_children(|parent| {
        parent.spawn((
            Node {
                min_width: px(350),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                padding: UiRect::axes(px(10), px(5)),
                ..default()
            },
            MainMenuTag,
            BackgroundColor(WINDOW_COLOR),
            children![
                (
                    Node {
                        margin: UiRect::vertical(px(10)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![ make_text(
                        Text::new("N-Body Simulation"),
                        Some(FontSize::Px(28.0)),
                        None
                    )]
                ),
                (make_button(Text::new("New Scene")), MainMenuButtonType::NewScene),
                (make_button(Text::new("About")), MainMenuButtonType::About),
                (make_button(Text::new("Exit")), MainMenuButtonType::Exit),
            ]
        ));
    });
}

fn update_main_menu(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &MainMenuButtonType,
        ),
        Changed<Interaction>
    >,
    mut next_state: ResMut<NextState<MainMenuState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, mut color, button_type) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                match button_type {
                    MainMenuButtonType::NewScene => { next_state.set(MainMenuState::NewScene); },
                    MainMenuButtonType::About => { next_state.set(MainMenuState::About); },
                    MainMenuButtonType::Exit => { exit.write(AppExit::Success); },
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

fn destroy_main_menu(
    mut commands: Commands,
    ui_items: Query<Entity, With<MainMenuTag>>,
) {
    for entity in ui_items {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
struct AboutMenuTag;

fn create_about_menu(
    mut commands: Commands,
    ui_data: Res<UIData>,
) {
    commands.entity(ui_data.main_ui_entity).with_children(|parent| {
        parent.spawn((
            Node {
                min_width: px(350),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                padding: UiRect::axes(px(350 * 5 / 100), px(5)),
                ..default()
            },
            AboutMenuTag,
            BackgroundColor(WINDOW_COLOR),
            children![
                (
                    Node {
                        margin: UiRect::vertical(px(10)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![ make_text(
                        Text::new("About N-Body Simulation"),
                        Some(FontSize::Px(28.0)),
                        None,
                    )],
                ),
                (
                    Button,
                    Node {
                        border: UiRect::axes(px(20), px(10)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![ make_text(
                        Text::new("Press ESC to go back"),
                        Some(FontSize::Px(14.0)),
                        Some(TEXT_COLOR_LIGHT),
                    )],
                ),
            ],
        ));
    });
}

fn update_about_menu(
    mut interaction_query: Query<
        (
            &Interaction,
            &Children,
        ),
        Changed<Interaction>
    >,
    mut text_query: Query<&mut TextColor>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<MainMenuState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(MainMenuState::MainMenu);
    }

    let Ok((interaction, children)) = &mut interaction_query.single_mut() else { return; };
    let mut color = text_query.get_mut(children[0]).unwrap();

    match *interaction {
        Interaction::Pressed => {
            next_state.set(MainMenuState::MainMenu);

        },
        Interaction::Hovered => {
            *color = TEXT_COLOR_LIGHT_HOVER.into();
        },
        Interaction::None => {
            *color = TEXT_COLOR_LIGHT.into();
        }
    }
}

fn destroy_about_menu(
    mut commands: Commands,
    ui_items: Query<Entity, With<AboutMenuTag>>,
) {
    for entity in ui_items {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
struct NewSceneMenuTag;

#[derive(Component)]
pub enum NewSceneButtonType {
    Start,
    Back,
}

fn create_new_scene_menu(
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
                        min_width: px(150),
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
                            Text::new("New Scene Settings"),
                            Some(FontSize::Px(24.0)),
                            None,
                        ),
                        make_checkbox(Text::new("Checkbox 0")),
                        make_checkbox(Text::new("Checkbox 1")),
                        make_flex_grow(1.0),
                        (make_button(Text::new("Start Simulation")), NewSceneButtonType::Start),
                        (make_button(Text::new("Back to Main Menu")), NewSceneButtonType::Back),
                    ],
                ),
                (
                    Node {
                        min_width: px(200),
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
                                padding: UiRect::all(px(5)),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Start,
                                justify_content: JustifyContent::FlexStart,
                                ..default()
                            },
                            BackgroundColor(WINDOW_COLOR_LIGHT),
                            children![
                                make_text(Text::new("Scenario 0"), None, None),
                                make_text(Text::new("Scenario 1"), None, None),
                                make_text(Text::new("Scenario 2"), None, None)
                            ],
                        )
                    ],
                ),
            ],
        ));
    });
}

fn update_new_scene_menu(
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

fn destroy_new_scene_menu(
    mut commands: Commands,
    ui_items: Query<Entity, With<NewSceneMenuTag>>,
) {
    for entity in ui_items {
        commands.entity(entity).despawn();
    }
}
