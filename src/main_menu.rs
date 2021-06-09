use bevy::prelude::*;

use crate::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(AppState::MainMenu).with_system(menu_setup.system()),
        )
        .add_system_set(SystemSet::on_update(AppState::MainMenu).with_system(menu_update.system()));
    }
}

fn menu_setup(mut commands: Commands, ui_resources: Res<UIResources>) {
    commands.spawn_bundle(UiCameraBundle::default());

    let button_bundle = ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(200.0), Val::Px(60.0)),
            margin: Rect::all(Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        ..Default::default()
    };

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: ui_resources.transparent.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(button_bundle.clone())
                .insert(ButtonType::Play)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Play",
                            TextStyle {
                                font: ui_resources.font.clone(),
                                font_size: 40.0,
                                color: Color::BLACK,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });

            parent
                .spawn_bundle(button_bundle.clone())
                .insert(ButtonType::Quit)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Quit",
                            TextStyle {
                                font: ui_resources.font.clone(),
                                font_size: 40.0,
                                color: Color::BLACK,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
        });
}

fn menu_update(
    mut state: ResMut<State<AppState>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut button_query: Query<(&Interaction, &ButtonType), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, button_type) in button_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                match button_type {
                    ButtonType::Play => state
                        .set(AppState::Cleanup(Box::new(AppState::InGame)))
                        .unwrap(),
                    ButtonType::Quit => app_exit_events.send(AppExit),
                    _ => (),
                };
            }
            _ => (),
        }
    }
}
