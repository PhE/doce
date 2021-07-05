use bevy::prelude::*;
use bevy_egui::{
    egui::{self, widgets::Widget},
    EguiContext,
};
use libp2p::gossipsub::GossipsubEvent;

use crate::network::{NetworkEvent, NetworkManager, NetworkTopic};

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
                SystemSet::on_update(AppState::Lobby)
                    .with_system(receive_chat_messages.system())
                    .with_system(lobby_ui.system().label("lobby_ui"))
                    .with_system(handle_lobby_ui_events.system().after("lobby_ui")),
            );
    }
}

struct LobbyUI {
    current_chat_message: String,
    chat_messages: Vec<String>,
}

enum LobbyUIEvent {
    SendChatMessage(String),
}

fn lobby_ui(
    egui_context: Res<EguiContext>,
    mut lobby_ui: ResMut<LobbyUI>,
    mut lobby_ui_events: EventWriter<LobbyUIEvent>,
) {
    egui::SidePanel::left("lobby", 400.0).show(egui_context.ctx(), |ui| {
        ui.horizontal(|ui| {
            ui.heading("Lobby");
            ui.button("Leave");
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Player 1");
            ui.checkbox(&mut true, "Ready");
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.set_enabled(false);
            ui.label("Player 2");
            ui.checkbox(&mut true, "Ready");
        });

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
                    for message in &lobby_ui.chat_messages {
                        ui.label(message);
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
                lobby_ui.chat_messages.push(message.clone());
                lobby_ui_events.send(LobbyUIEvent::SendChatMessage(message));
            }
        });
    });
}

fn handle_lobby_ui_events(
    mut lobby_ui_events: EventReader<LobbyUIEvent>,
    mut network_manager: ResMut<NetworkManager>,
) {
    for event in lobby_ui_events.iter() {
        match event {
            LobbyUIEvent::SendChatMessage(message) => {
                network_manager.publish(NetworkTopic::new("chat"), message.as_bytes());
            }
        }
    }
}

fn receive_chat_messages(
    mut lobby_ui: ResMut<LobbyUI>,
    mut network_events: EventReader<NetworkEvent>,
) {
    for event in network_events.iter() {
        if let NetworkEvent::Behaviour(GossipsubEvent::Message { message, .. }) = event {
            if NetworkTopic::new("chat").hash() == message.topic {
                lobby_ui
                    .chat_messages
                    .push(String::from_utf8(message.data.clone()).unwrap());
            }
        }
    }
}
