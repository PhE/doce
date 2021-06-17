use bevy::prelude::*;

pub struct InitSoundPlugin;

impl Plugin for InitSoundPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, init_sound_resources.system());
    }
}

pub struct SoundResources {
    pub pistol_shoot: Handle<AudioSource>,
    pub pistol_reload: Handle<AudioSource>,
}

fn init_sound_resources(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SoundResources {
        pistol_shoot: asset_server.load("sounds/pistol_shoot.mp3"),
        pistol_reload: asset_server.load("sounds/pistol_reload.mp3"),
    });
}
