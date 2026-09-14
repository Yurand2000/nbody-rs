use bevy::prelude::*;
use crate::common::*;

mod about_menu;
mod new_scene_menu;

use about_menu::*;
use new_scene_menu::*;

pub struct MainMenuPlugin;

#[derive(Resource)]
pub struct UIData {
    camera_entity: Entity,
    main_ui_entity: Entity,
}

impl Plugin for MainMenuPlugin {
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
    mut next_state: ResMut<NextState<GameSceneState>>,
) {
    use bevy::camera::visibility::RenderLayers;

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

    next_state.set(GameSceneState::MainMenu);
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(GameSceneState = GameSceneState::MainMenu)]
pub enum MainMenuState {
    #[default]
    MainMenu,
    NewScene,
    About,
}

pub const WINDOW_COLOR: Color = Color::srgba(0.2, 0.2, 0.2, 0.7);
pub const WINDOW_COLOR_LIGHT: Color = Color::srgba(0.25, 0.25, 0.25, 0.7);
pub const BORDER_COLOR: Color = Color::BLACK;
pub const BUTTON_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
pub const BUTTON_COLOR_HOVER: Color = Color::srgb(0.5, 0.5, 0.5);
pub const BUTTON_COLOR_SELECTED: Color = Color::srgb(0.4, 0.4, 0.4);
pub const TEXT_COLOR: Color = Color::WHITE;
pub const TEXT_COLOR_LIGHT: Color = Color::srgb(0.4, 0.4, 0.4);
pub const TEXT_COLOR_LIGHT_HOVER: Color = Color::srgb(0.6, 0.6, 0.6);
pub const CHECKBOX_TRUE: Color = BUTTON_COLOR_SELECTED;
pub const CHECKBOX_FALSE: Color = Color::NONE;

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
    text: impl Bundle
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
        children![ text ]
    )
}

fn make_list_button(
    text: impl Bundle
) -> impl Bundle {
    (
        Button,
        Node {
            width: percent(100.0),
            height: auto(),
            border: UiRect::all(px(2)),
            margin: UiRect::vertical(px(2)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(5)),
            ..default()
        },
        BackgroundColor(BUTTON_COLOR),
        children![ text ]
    )
}

fn make_toggle_button(
    text: impl Bundle
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
                    border_radius: BorderRadius::MAX,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    ..default()
                },
                BorderColor::all(BORDER_COLOR),
                BackgroundColor(BUTTON_COLOR),
                children![
                    (
                        Node {
                            width: px(12),
                            height: px(12),
                            margin: UiRect::all(px(3)),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(BUTTON_COLOR_HOVER),
                        Visibility::Inherited,
                    ),
                    (
                        Node {
                            width: px(12),
                            height: px(12),
                            margin: UiRect::all(px(3)),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(BUTTON_COLOR_HOVER),
                        Visibility::Hidden,
                    ),
                ]
            ),
            (
                Node {
                    border: UiRect::axes(px(10), px(2)),
                    margin: UiRect::vertical(px(2)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![ text ]
            )
        ]

    )
}

fn make_radio_button(
    text: impl Bundle
) -> impl Bundle {
    (
        bevy::ui_widgets::Checkbox,
        Node {
            border: UiRect::all(px(2)),
            margin: UiRect::vertical(px(2)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![
            (
                Node {
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BorderColor::all(BORDER_COLOR),
                BackgroundColor(BUTTON_COLOR),
                children![(
                    Node {
                        width: px(8),
                        height: px(8),
                        margin: UiRect::all(px(3)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(CHECKBOX_TRUE),
                )]
            ),
            (
                Node {
                    // height: px(20),
                    border: UiRect::axes(px(10), px(2)),
                    margin: UiRect::vertical(px(2)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    justify_content: JustifyContent::SpaceEvenly,
                    ..default()
                },
                children![ text ]
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
