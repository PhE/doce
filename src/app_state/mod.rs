use bevy::prelude::*;

use self::lobby::LobbyPlugin;

pub mod lobby;

pub struct InitAppStatePlugin(pub AppState);

impl Plugin for InitAppStatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        // app.add_state(self.0);
        app
            .add_plugin(LobbyPlugin)
            .insert_resource(State::new(self.0))
            // .add_system_set_to_stage(CoreStage::First, State::<AppState>::get_driver())
            // .add_system_set_to_stage(CoreStage::PreUpdate, State::<AppState>::get_driver())
            .add_system_set_to_stage(CoreStage::Update, State::<AppState>::get_driver())
            // .add_system_set_to_stage(CoreStage::PostUpdate, State::<AppState>::get_driver())
            // .add_system_set_to_stage(CoreStage::Last, State::<AppState>::get_driver())
            ;
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AppState {
    Cleanup,
    InGame,
    GameOver,
    MainMenu,
    Replay,
    // Lobby
    InLobby,
    JoiningLobby,
}
