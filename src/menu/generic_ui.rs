use bevy::{picking::hover::Hovered, ui::Checked};
use super::*;

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

pub fn make_text(
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

#[derive(Component)]
pub struct StyledButton;

pub fn make_button(
    text: impl Bundle
) -> impl Bundle {
    (
        Button,
        StyledButton,
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

#[derive(Component)]
pub struct StyledTextButton;

pub fn make_text_button(
    text: impl Into<String>,
) -> impl Bundle {
    (
        Button,
        StyledTextButton,
        Node {
            border: UiRect::axes(px(20), px(10)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![ make_text(
            Text::new(text),
            Some(FontSize::Px(14.0)),
            Some(TEXT_COLOR_LIGHT),
        )],
    )
}

pub fn make_list_button(
    text: impl Bundle
) -> impl Bundle {
    (
        Button,
        StyledButton,
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

#[derive(Component)]
pub struct StyledToggleButton;

pub fn make_toggle_button(
    text: impl Bundle
) -> impl Bundle {
    (
        Button,
        StyledToggleButton,
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

pub fn make_radio_button(
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

pub fn make_text_input<'a>(
    initial_text: &'a str,
    text: impl Bundle,
    editor: impl Bundle,
) -> impl Bundle {
    (
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
                    border: px(2).all(),
                    ..default()
                },
                BorderColor::all(BORDER_COLOR),
                {
                    let mut e = bevy::text::EditableText::new(initial_text);
                    e.visible_width = Some(4.0);
                    e.allow_newlines = false;
                    e
                },
                TextLayout::no_wrap(),
                bevy::text::TextCursorStyle::default(),
                BackgroundColor(BUTTON_COLOR),
                editor,
            ),
            (
                Node {
                    border: UiRect::axes(px(10), px(2)),
                    margin: UiRect::vertical(px(2)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    justify_content: JustifyContent::SpaceEvenly,
                    ..default()
                },
                children![ text ]
            ),
        ]
    )
}

pub fn make_flex_grow(
    amount: f32
) -> impl Bundle {
    Node {
        flex_grow: amount,
        ..default()
    }
}

// Systems
pub fn update_buttons_hover(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<StyledButton>)>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => (),
            Interaction::Hovered => {
                *color = BUTTON_COLOR_HOVER.into();
            },
            Interaction::None => {
                *color = BUTTON_COLOR.into();
            }
        }
    }
}

pub fn update_text_buttons_hover(
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<StyledTextButton>)>,
    mut text_query: Query<&mut TextColor>,
) {
    for (interaction, children) in &mut interaction_query {
        let mut color = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => (),
            Interaction::Hovered => {
                *color = TEXT_COLOR_LIGHT_HOVER.into();
            },
            Interaction::None => {
                *color = TEXT_COLOR_LIGHT.into();
            }
        }
    }
}

pub fn update_toggle_buttons_style(
    mut commands: Commands,
    interaction_query: Query<(Entity, Has<Checked>, &Interaction, &Children), (Changed<Interaction>, With<StyledToggleButton>)>,
    mut children_query: Query<(&Children, &mut BackgroundColor), Without<StyledToggleButton>>,
    mut visibility_query: Query<&mut Visibility, Without<StyledToggleButton>>,
) {
    for (entity, checked, interaction, children) in interaction_query {
        let (children, mut color) = children_query.get_mut(children[0]).unwrap();

        match interaction {
            Interaction::Pressed => {
                let off_entity = children[0];
                let on_entity = children[1];

                if checked {
                    *visibility_query.get_mut(off_entity).unwrap() = Visibility::Inherited;
                    *visibility_query.get_mut(on_entity).unwrap() = Visibility::Hidden;
                    commands.entity(entity).remove::<Checked>();
                } else {
                    *visibility_query.get_mut(off_entity).unwrap() = Visibility::Hidden;
                    *visibility_query.get_mut(on_entity).unwrap() = Visibility::Inherited;
                    commands.entity(entity).insert(Checked);
                }
            },
            Interaction::Hovered => {
                *color = BackgroundColor(BUTTON_COLOR_SELECTED);
            },
            Interaction::None => {
                *color = BackgroundColor(BUTTON_COLOR);
            },
        }
    }
}
