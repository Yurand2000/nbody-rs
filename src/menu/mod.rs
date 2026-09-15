use bevy::prelude::*;
use crate::common::*;

mod about_menu;
mod new_scene_menu;
pub mod generic_ui;

use about_menu::*;
use new_scene_menu::*;
pub use generic_ui::*;

pub struct MainMenuPlugin;

#[derive(Resource)]
pub struct UIData {
    main_ui_entity: Entity,
}

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_sub_state::<MainMenuState>()
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (
                update_buttons_hover,
                update_text_buttons_hover,
                update_list_buttons_style,
                update_list_buttons_style_2,
                update_toggle_buttons_style
            ))

            .add_systems(OnEnter(MainMenuState::MainMenu), create_main_menu)
            .add_systems(Update, update_main_menu.run_if(in_state(MainMenuState::MainMenu)))
            .add_systems(OnExit(MainMenuState::MainMenu), destroy_main_menu)

            .add_systems(OnEnter(MainMenuState::About), create_about_menu)
            .add_systems(Update, update_about_menu.run_if(in_state(MainMenuState::About)))
            .add_systems(OnExit(MainMenuState::About), destroy_about_menu)

            .add_systems(OnEnter(MainMenuState::NewScene), create_new_scene_menu)
            .add_systems(Update, (on_start_button, on_back_button, on_cuda_toggle_button)
                .run_if(in_state(MainMenuState::NewScene)))
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
        UIData { main_ui_entity }
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
    interaction_query: Query<(&Interaction, &MainMenuButtonType), Changed<Interaction>>,
    mut next_state: ResMut<NextState<MainMenuState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button_type) in interaction_query {
        match *interaction {
            Interaction::Pressed => {
                match button_type {
                    MainMenuButtonType::NewScene => { next_state.set(MainMenuState::NewScene); },
                    MainMenuButtonType::About => { next_state.set(MainMenuState::About); },
                    MainMenuButtonType::Exit => { exit.write(AppExit::Success); },
                };
            },
            _ => (),
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
