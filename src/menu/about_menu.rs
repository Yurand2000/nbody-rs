use super::*;

#[derive(Component)]
pub struct AboutMenuTag;

pub fn create_about_menu(
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
                make_text_button("Press ESC to go back"),
            ],
        ));
    });
}

pub fn update_about_menu(
    mut interaction_query: Query<&Interaction , Changed<Interaction>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<MainMenuState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(MainMenuState::MainMenu);
    }

    let Ok(interaction) = &mut interaction_query.single_mut() else { return; };

    match *interaction {
        Interaction::Pressed => {
            next_state.set(MainMenuState::MainMenu);
        },
        _ => (),
    }
}

pub fn destroy_about_menu(
    mut commands: Commands,
    ui_items: Query<Entity, With<AboutMenuTag>>,
) {
    for entity in ui_items {
        commands.entity(entity).despawn();
    }
}

