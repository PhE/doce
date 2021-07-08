use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContext};
use fake::{faker, Fake};

use crate::{
    app_state::AppState,
    cleanup::CleanupConfig,
    network::{NetworkAddress, NetworkManager, NetworkTopic},
    party::Party,
    player::{Player, PlayerId},
    resources::UIResources,
};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(MainMenuState::default())
            .add_system_set(
                SystemSet::on_enter(AppState::MainMenu).with_system(menu_setup.system()),
            )
            .add_system_set(
                SystemSet::on_update(AppState::MainMenu)
                    .with_system(main_menu_dialog.system().before("menu_update"))
                    .with_system(menu_update.system().label("menu_update")),
            );
    }
}

struct MainMenuState {
    current_dialog: Option<MainMenuDialog>,
    player_name: String,
    party_address: String,
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self {
            current_dialog: None,
            player_name: faker::name::en::Name().fake(),
            party_address: "".into(),
        }
    }
}

enum MainMenuButton {
    CreateLobby,
    JoinLobby,
    ChangeName,
    Quit,
}

enum MainMenuDialog {
    JoinLobby,
    ChangeName,
}

fn menu_setup(mut commands: Commands, ui_resources: Res<UIResources>) {
    commands.spawn_bundle(UiCameraBundle::default());

    let button_bundle = ButtonBundle {
        style: Style {
            padding: Rect {
                top: Val::Px(5.0),
                bottom: Val::Px(5.0),
                left: Val::Px(10.0),
                right: Val::Px(10.0),
            },
            margin: Rect::all(Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        ..Default::default()
    };

    let create_button =
        |parent: &mut ChildBuilder, button_type: MainMenuButton, button_name: &str| {
            parent
                .spawn_bundle(button_bundle.clone())
                .insert(button_type)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            button_name,
                            TextStyle {
                                font: ui_resources.font.clone(),
                                font_size: 64.0,
                                color: Color::BLACK,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
        };

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: Rect::all(Val::Auto),
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Stretch,
                ..Default::default()
            },
            material: ui_resources.transparent.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            create_button(parent, MainMenuButton::CreateLobby, "Create Lobby");
            create_button(parent, MainMenuButton::JoinLobby, "Join Lobby");
            create_button(parent, MainMenuButton::ChangeName, "Change Name");
            create_button(parent, MainMenuButton::Quit, "Quit");
        });
}

fn menu_update(
    mut commands: Commands,
    mut main_menu_state: ResMut<MainMenuState>,
    mut network_manager: ResMut<NetworkManager>,
    mut app_state: ResMut<State<AppState>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut cleanup_config: ResMut<CleanupConfig>,
    query: Query<(&Interaction, &MainMenuButton), (Changed<Interaction>, With<Button>)>,
) {
    if main_menu_state.current_dialog.is_some() {
        return;
    }

    for (interaction, button) in query.iter() {
        if let Interaction::Clicked = interaction {
            match button {
                MainMenuButton::CreateLobby => {
                    create_lobby(
                        &mut commands,
                        &mut network_manager,
                        &mut app_state,
                        &mut cleanup_config,
                        main_menu_state.player_name.clone(),
                    );
                }
                MainMenuButton::JoinLobby => {
                    main_menu_state.current_dialog = Some(MainMenuDialog::JoinLobby);
                }
                MainMenuButton::ChangeName => {
                    main_menu_state.current_dialog = Some(MainMenuDialog::ChangeName);
                }
                MainMenuButton::Quit => app_exit_events.send(AppExit),
            };
        }
    }
}

fn main_menu_dialog(
    mut commands: Commands,
    egui_context: Res<EguiContext>,
    mut main_menu_state: ResMut<MainMenuState>,
    mut network_manager: ResMut<NetworkManager>,
    mut app_state: ResMut<State<AppState>>,
    mut cleanup_config: ResMut<CleanupConfig>,
) {
    let mut close_dialog = false;

    match main_menu_state.current_dialog {
        Some(MainMenuDialog::JoinLobby) => {
            egui::Window::new("Join Lobby")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .show(egui_context.ctx(), |ui| {
                    let mut join = false;

                    ui.horizontal(|ui| {
                        ui.label("Address:");

                        let text_edit_lost_focus = ui
                            .text_edit_singleline(&mut main_menu_state.party_address)
                            .lost_focus();

                        join = text_edit_lost_focus && ui.input().key_pressed(egui::Key::Enter);
                    });

                    ui.horizontal(|ui| {
                        ui.scope(|ui| {
                            let address = main_menu_state.party_address.parse();
                            let address_is_valid =
                                !main_menu_state.party_address.is_empty() && address.is_ok();

                            ui.set_enabled(address_is_valid);

                            let button_clicked = ui.button("Join").clicked();

                            if address_is_valid && (button_clicked || join) {
                                join_lobby(
                                    &mut commands,
                                    &mut network_manager,
                                    &mut app_state,
                                    &mut cleanup_config,
                                    main_menu_state.player_name.clone(),
                                    address.unwrap(),
                                );
                            }
                        });

                        close_dialog = ui.button("Cancel").clicked();
                    });
                });
        }
        Some(MainMenuDialog::ChangeName) => {
            egui::Window::new("Change Name")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .show(egui_context.ctx(), |ui| {
                    let text_edit_lost_focus = ui
                        .text_edit_singleline(&mut main_menu_state.player_name)
                        .lost_focus();

                    let button_clicked = ui.button("Confirm").clicked();

                    close_dialog = button_clicked
                        || text_edit_lost_focus && ui.input().key_pressed(egui::Key::Enter);
                });
        }
        None => (),
    };

    if close_dialog {
        main_menu_state.current_dialog = None;
    }
}

fn create_lobby(
    commands: &mut Commands,
    network_manager: &mut ResMut<NetworkManager>,
    app_state: &mut ResMut<State<AppState>>,
    cleanup_config: &mut ResMut<CleanupConfig>,
    player_name: String,
) {
    let player = Player {
        id: PlayerId::new(network_manager.local_peer_id()),
        name: player_name,
    };

    commands.insert_resource(Party::new(player));

    network_manager
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .unwrap();
    network_manager.subscribe(NetworkTopic::new("join_request"));

    cleanup_config.next_state_after_cleanup = Some(AppState::InLobby);
    app_state.set(AppState::Cleanup).unwrap();
}

fn join_lobby(
    commands: &mut Commands,
    network_manager: &mut ResMut<NetworkManager>,
    app_state: &mut ResMut<State<AppState>>,
    cleanup_config: &mut ResMut<CleanupConfig>,
    player_name: String,
    address: NetworkAddress,
) {
    let player = Player {
        id: PlayerId::new(network_manager.local_peer_id()),
        name: player_name,
    };

    commands.insert_resource(Party::new(player));

    network_manager
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .unwrap();
    network_manager.subscribe(NetworkTopic::new("join_accepted"));
    network_manager.subscribe(NetworkTopic::new("join_rejected"));
    network_manager.dial_addr(address);

    cleanup_config.next_state_after_cleanup = Some(AppState::JoiningLobby);
    app_state.set(AppState::Cleanup).unwrap();
}
