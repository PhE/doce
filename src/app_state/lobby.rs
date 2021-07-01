use bevy::prelude::*;

use crate::resources::UIResources;

use super::AppState;

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(SystemSet::on_enter(AppState::Lobby).with_system(setup_ui.system()));
    }
}

struct LobbyPlayerIndex(usize);

struct LobbyPlayerState {
    ready: bool,
}

fn setup_ui(mut commands: Commands, ui_resources: Res<UIResources>) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                ..Default::default()
            },
            material: ui_resources.black.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::ColumnReverse,
                        ..Default::default()
                    },
                    material: ui_resources.transparent.clone(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    for i in 0..4 {
                        parent
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    margin: Rect::all(Val::Px(10.0)),
                                    ..Default::default()
                                },
                                material: ui_resources.white.clone(),
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                parent.spawn_bundle(TextBundle {
                                    text: Text::with_section(
                                        format!("Player_{}", i),
                                        TextStyle {
                                            font: ui_resources.font.clone(),
                                            font_size: 24.0,
                                            color: Color::BLACK,
                                        },
                                        Default::default(),
                                    ),
                                    ..Default::default()
                                });

                                parent.spawn_bundle(TextBundle {
                                    style: Style {
                                        margin: Rect {
                                            left: Val::Auto,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    text: Text::with_section(
                                        "READY",
                                        TextStyle {
                                            font: ui_resources.font.clone(),
                                            font_size: 16.0,
                                            color: Color::GREEN,
                                        },
                                        Default::default(),
                                    ),
                                    ..Default::default()
                                });
                            });
                    }

                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::FlexEnd,
                                ..Default::default()
                            },
                            material: ui_resources.transparent.clone(),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn_bundle(ButtonBundle {
                                    style: Style {
                                        margin: Rect {
                                            top: Val::Px(10.0),
                                            bottom: Val::Px(10.0),
                                            left: Val::Px(10.0),
                                            right: Val::Auto,
                                        },
                                        padding: Rect {
                                            top: Val::Px(5.0),
                                            bottom: Val::Px(5.0),
                                            left: Val::Px(10.0),
                                            right: Val::Px(10.0),
                                        },
                                        ..Default::default()
                                    },
                                    material: ui_resources.white.clone(),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent.spawn_bundle(TextBundle {
                                        text: Text::with_section(
                                            "LEAVE",
                                            TextStyle {
                                                font: ui_resources.font.clone(),
                                                font_size: 16.0,
                                                color: Color::RED,
                                            },
                                            Default::default(),
                                        ),
                                        ..Default::default()
                                    });
                                });

                            parent
                                .spawn_bundle(ButtonBundle {
                                    style: Style {
                                        margin: Rect::all(Val::Px(10.0)),
                                        padding: Rect {
                                            top: Val::Px(5.0),
                                            bottom: Val::Px(5.0),
                                            left: Val::Px(10.0),
                                            right: Val::Px(10.0),
                                        },
                                        ..Default::default()
                                    },
                                    material: ui_resources.white.clone(),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent.spawn_bundle(TextBundle {
                                        text: Text::with_section(
                                            "READY",
                                            TextStyle {
                                                font: ui_resources.font.clone(),
                                                font_size: 16.0,
                                                color: Color::BLACK,
                                            },
                                            Default::default(),
                                        ),
                                        ..Default::default()
                                    });
                                });

                            parent
                                .spawn_bundle(ButtonBundle {
                                    style: Style {
                                        margin: Rect::all(Val::Px(10.0)),
                                        padding: Rect {
                                            top: Val::Px(5.0),
                                            bottom: Val::Px(5.0),
                                            left: Val::Px(10.0),
                                            right: Val::Px(10.0),
                                        },
                                        ..Default::default()
                                    },
                                    material: ui_resources.white.clone(),
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent.spawn_bundle(TextBundle {
                                        text: Text::with_section(
                                            "START",
                                            TextStyle {
                                                font: ui_resources.font.clone(),
                                                font_size: 16.0,
                                                color: Color::BLACK,
                                            },
                                            Default::default(),
                                        ),
                                        ..Default::default()
                                    });
                                });
                        });
                });
        });
}
