use bevy::{app::AppExit, prelude::*};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

use crate::{
    app_state::AppState,
    cleanup::CleanupConfig,
    network::{NetworkAddress, NetworkManager, NetworkTopic},
    party::Party,
    player::Player,
    resources::UIResources,
};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(MainMenuState {
            player_name: format!("{:0x}", rand::random::<u64>()),
            party_address: "".into(),
        })
        .add_plugin(InspectorPlugin::<MainMenuState>::new_insert_manually())
        .add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(menu_setup.system()))
        .add_system_set(
            SystemSet::on_update(AppState::MainMenu)
                .with_system(update_join_lobby_button.system().before("menu_update"))
                .with_system(menu_update.system().label("menu_update")),
        );
    }
}

#[derive(Inspectable)]
struct MainMenuState {
    player_name: String,
    party_address: String,
}

enum MainMenuButton {
    CreateLobby,
    JoinLobby,
    Quit,
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
                .insert(MainMenuButton::CreateLobby)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Create Lobby",
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
                .insert(MainMenuButton::JoinLobby)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Join Lobby",
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
                .insert(MainMenuButton::Quit)
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

fn update_join_lobby_button(
    main_menu_state: Res<MainMenuState>,
    mut text_query: Query<&mut Text>,
    query: Query<(&MainMenuButton, &Children, &mut Style)>,
) {
    query.for_each_mut(|(button, children, mut style)| {
        if let MainMenuButton::JoinLobby = button {
            let is_visible = !main_menu_state.party_address.is_empty()
                && main_menu_state
                    .party_address
                    .parse::<NetworkAddress>()
                    .is_ok();

            style.display = if is_visible {
                Display::Flex
            } else {
                Display::None
            };

            for child in children.iter() {
                let mut text = text_query.get_mut(*child).unwrap();

                for section in &mut text.sections {
                    section
                        .style
                        .color
                        .set_a(if is_visible { 1.0 } else { 0.0 });
                }
            }
        }
    });
}

fn menu_update(
    mut commands: Commands,
    main_menu_state: Res<MainMenuState>,
    mut network_manager: ResMut<NetworkManager>,
    mut state: ResMut<State<AppState>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut cleanup_config: ResMut<CleanupConfig>,
    query: Query<(&Interaction, &MainMenuButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, button) in query.iter() {
        match interaction {
            Interaction::Clicked => {
                match button {
                    MainMenuButton::CreateLobby => {
                        let player = Player {
                            name: main_menu_state.player_name.clone(),
                        };

                        commands.insert_resource(Party::new(player));

                        network_manager.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();
                        network_manager.subscribe(NetworkTopic::new("chat"));
                        network_manager.subscribe(NetworkTopic::new("joined"));

                        cleanup_config.next_state_after_cleanup = Some(AppState::Lobby);
                        state.set(AppState::Cleanup).unwrap();
                    }
                    MainMenuButton::JoinLobby => {
                        let player = Player {
                            name: main_menu_state.player_name.clone(),
                        };

                        commands.insert_resource(Party::new(player));

                        network_manager.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();
                        network_manager.subscribe(NetworkTopic::new("chat"));
                        network_manager.subscribe(NetworkTopic::new("joining"));
                        network_manager.dial(main_menu_state.party_address.parse().unwrap());

                        cleanup_config.next_state_after_cleanup = Some(AppState::Lobby);
                        state.set(AppState::Cleanup).unwrap();
                    }
                    MainMenuButton::Quit => app_exit_events.send(AppExit),
                };
            }
            _ => (),
        }
    }
}
