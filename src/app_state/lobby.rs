use std::{collections::HashMap, ops::Deref};

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, widgets::Widget},
    EguiContext,
};
use libp2p::gossipsub::GossipsubEvent;

use crate::{
    cleanup::CleanupConfig,
    network::{NetworkBehaviourEvent, NetworkEvent, NetworkManager, NetworkTopic},
    party::Party,
    player::{Player, PlayerId},
};

use super::AppState;

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<LobbyUIEvent>()
            .insert_resource(LobbyUI {
                current_chat_message: "".into(),
                chat_messages: Vec::new(),
            })
            .add_system_set(
                SystemSet::on_update(AppState::JoiningLobby)
                    .with_system(handle_join_lobby_events.system()),
            )
            .add_system_set(
                SystemSet::on_enter(AppState::InLobby).with_system(setup_lobby.system()),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InLobby)
                    .with_system(handle_host_events.system())
                    .with_system(handle_client_events.system())
                    .with_system(receive_chat_messages.system())
                    .with_system(lobby_ui.system().label("lobby_ui"))
                    .with_system(handle_lobby_ui_events.system().after("lobby_ui")),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::InLobby).with_system(unsubscribe_all_topics.system()),
            );
    }
}

struct Lobby {
    player_states: HashMap<PlayerId, LobbyPlayerState>,
}

struct LobbyPlayerState {
    ready: bool,
}

struct LobbyChatMessage {
    player_name: String,
    message: String,
}

struct LobbyUI {
    current_chat_message: String,
    chat_messages: Vec<LobbyChatMessage>,
}

enum LobbyUIEvent {
    Leave,
    SendChatMessage(String),
}

fn handle_join_lobby_events(
    mut cleanup_config: ResMut<CleanupConfig>,
    mut app_state: ResMut<State<AppState>>,
    mut party: ResMut<Party>,
    mut network_manager: ResMut<NetworkManager>,
    mut network_events: EventReader<NetworkEvent>,
) {
    for event in network_events.iter() {
        match event {
            NetworkEvent::ConnectionEstablished { .. } => {
                let data = serde_json::to_vec(&party.players[&party.host_id]).unwrap();
                network_manager.publish(NetworkTopic::new("join_request"), data);
            }
            NetworkEvent::UnknownPeerUnreachableAddr { address, error } => {
                error!("Cannot connect to {:?}: {:?}", address, error);
                cleanup_config.next_state_after_cleanup = Some(AppState::MainMenu);
                app_state.set(AppState::Cleanup).unwrap();
            }
            NetworkEvent::Behaviour(NetworkBehaviourEvent::Gossipsub(
                GossipsubEvent::Message { message, .. },
            )) => {
                if NetworkTopic::new("join_accepted").hash() == message.topic {
                    match serde_json::from_slice::<'_, Party>(&message.data) {
                        Ok(new_party) => {
                            let host_id = party.host_id;
                            let player = party.players.remove(&host_id).unwrap();
                            *party = new_party;
                            party.players.insert(player.id, player);
                            network_manager.unsubscribe(NetworkTopic::new("join_accepted"));
                            network_manager.unsubscribe(NetworkTopic::new("join_rejected"));
                            network_manager.subscribe(NetworkTopic::new("joined"));
                            app_state.set(AppState::InLobby).unwrap();
                        }
                        Err(error) => {
                            let error = format!(
                                "Cannot parse Party from join_accepted message: {:?}",
                                error
                            );
                            error!("{}", error);
                            cleanup_config.next_state_after_cleanup = Some(AppState::MainMenu);
                            app_state.set(AppState::Cleanup).unwrap();
                        }
                    };
                } else if NetworkTopic::new("join_rejected").hash() == message.topic {
                    let error = String::from_utf8_lossy(&message.data);
                    error!("Join rejected: {}", error);
                    cleanup_config.next_state_after_cleanup = Some(AppState::MainMenu);
                    app_state.set(AppState::Cleanup).unwrap();
                }
            }
            _ => (),
        }
    }
}

fn setup_lobby(mut network_manager: ResMut<NetworkManager>) {
    network_manager.subscribe(NetworkTopic::new("chat"));
}

fn lobby_ui(
    egui_context: Res<EguiContext>,
    party: Res<Party>,
    network_manager: Res<NetworkManager>,
    mut lobby_ui: ResMut<LobbyUI>,
    mut lobby_ui_events: EventWriter<LobbyUIEvent>,
) {
    egui::SidePanel::left("lobby", 400.0).show(egui_context.ctx(), |ui| {
        ui.horizontal(|ui| {
            ui.heading("Lobby");

            if ui.button("Leave").clicked() {
                lobby_ui_events.send(LobbyUIEvent::Leave);
            }
        });

        for player in party.players.values() {
            ui.separator();

            ui.horizontal(|ui| {
                ui.set_enabled(false);

                if player.id == party.host_id {
                    ui.label(
                        egui::Label::new("👑")
                            .strong()
                            .text_color(egui::Color32::YELLOW),
                    );
                }

                ui.label(&player.name);
                ui.checkbox(&mut false, "Ready");
            });
        }

        ui.separator();

        ui.horizontal(|ui| {
            egui::Button::new("Start Game").enabled(false).ui(ui);
        });

        ui.separator();

        let mut size = ui.available_size();
        size.y = f32::max(size.y - 50.0, 0.0);

        let (_, rect) = ui.allocate_space(size);

        ui.allocate_ui_at_rect(rect, |ui| {
            egui::ScrollArea::auto_sized()
                .always_show_scroll(true)
                .show(ui, |ui| {
                    for LobbyChatMessage {
                        player_name,
                        message,
                    } in &lobby_ui.chat_messages
                    {
                        ui.horizontal(|ui| {
                            ui.label(egui::Label::new(format!("{}:", player_name)).strong());
                            ui.label(message);
                        });
                    }
                });
        });

        ui.allocate_rect(rect, egui::Sense::hover());

        ui.separator();

        ui.with_layout(egui::Layout::right_to_left(), |ui| {
            let button_clicked = ui.button("Send").clicked();
            let text_edit_lost_focus =
                egui::TextEdit::singleline(&mut lobby_ui.current_chat_message)
                    .desired_width(ui.available_width())
                    .ui(ui)
                    .lost_focus();

            if button_clicked || (text_edit_lost_focus && ui.input().key_pressed(egui::Key::Enter))
            {
                let message = std::mem::replace(&mut lobby_ui.current_chat_message, "".into());
                lobby_ui.chat_messages.push(LobbyChatMessage {
                    player_name: party.players[&network_manager.local_peer_id().into()]
                        .name
                        .clone(),
                    message: message.clone(),
                });
                lobby_ui_events.send(LobbyUIEvent::SendChatMessage(message));
            }
        });
    });
}

fn handle_lobby_ui_events(
    mut cleanup_config: ResMut<CleanupConfig>,
    mut app_state: ResMut<State<AppState>>,
    mut lobby_ui_events: EventReader<LobbyUIEvent>,
    mut network_manager: ResMut<NetworkManager>,
) {
    for event in lobby_ui_events.iter() {
        match event {
            LobbyUIEvent::Leave => {
                cleanup_config.next_state_after_cleanup = Some(AppState::MainMenu);
                app_state.set(AppState::Cleanup).unwrap();
            }
            LobbyUIEvent::SendChatMessage(message) => {
                network_manager.publish(NetworkTopic::new("chat"), message.as_bytes());
            }
        }
    }
}

fn handle_host_events(
    mut party: ResMut<Party>,
    mut network_manager: ResMut<NetworkManager>,
    mut network_events: EventReader<NetworkEvent>,
) {
    if party.host_id != network_manager.local_peer_id().into() {
        return;
    }

    for event in network_events.iter() {
        if let NetworkEvent::Behaviour(NetworkBehaviourEvent::Gossipsub(
            GossipsubEvent::Message { message, .. },
        )) = event
        {
            if NetworkTopic::new("join_request").hash() == message.topic {
                match serde_json::from_slice::<'_, Player>(&message.data) {
                    Ok(player) => {
                        let party_json = serde_json::to_vec(party.deref()).unwrap();
                        party.players.insert(player.id, player);
                        network_manager.publish(NetworkTopic::new("join_accepted"), party_json);
                        network_manager.publish(NetworkTopic::new("joined"), message.data.clone());
                    }
                    Err(error) => {
                        let error =
                            format!("Cannot parse Player from join_request message: {:?}", error);
                        error!("{}", error);
                        network_manager.publish(NetworkTopic::new("join_rejected"), error);
                    }
                };
            }
        }
    }
}

fn handle_client_events(
    mut party: ResMut<Party>,
    network_manager: Res<NetworkManager>,
    mut network_events: EventReader<NetworkEvent>,
) {
    if party.host_id == network_manager.local_peer_id().into() {
        return;
    }

    for event in network_events.iter() {
        match event {
            NetworkEvent::Behaviour(NetworkBehaviourEvent::Gossipsub(
                GossipsubEvent::Message { message, .. },
            )) => {
                if NetworkTopic::new("joined").hash() == message.topic {
                    match serde_json::from_slice::<'_, Player>(&message.data) {
                        Ok(player) => {
                            party.players.insert(player.id, player);
                        }
                        Err(error) => {
                            let error = format!(
                                "Cannot parse Player from join_request message: {:?}",
                                error
                            );
                            error!("{}", error);
                        }
                    };
                }
            }
            _ => (),
        }
    }
}

fn receive_chat_messages(
    party: Res<Party>,
    mut lobby_ui: ResMut<LobbyUI>,
    mut network_events: EventReader<NetworkEvent>,
) {
    for event in network_events.iter() {
        if let NetworkEvent::Behaviour(NetworkBehaviourEvent::Gossipsub(
            GossipsubEvent::Message { message, .. },
        )) = event
        {
            if NetworkTopic::new("chat").hash() == message.topic {
                if let Some(player) = party.players.get(&message.source.unwrap().into()) {
                    lobby_ui.chat_messages.push(LobbyChatMessage {
                        player_name: player.name.clone(),
                        message: String::from_utf8(message.data.clone()).unwrap(),
                    });
                }
            }
        }
    }
}

fn unsubscribe_all_topics(mut network_manager: ResMut<NetworkManager>) {
    network_manager.unsubscribe(NetworkTopic::new("chat"));
    network_manager.unsubscribe(NetworkTopic::new("joined"));
    network_manager.unsubscribe(NetworkTopic::new("join_accepted"));
    network_manager.unsubscribe(NetworkTopic::new("join_rejected"));
    network_manager.unsubscribe(NetworkTopic::new("join_request"));
}
