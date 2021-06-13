use bevy::prelude::*;

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(CoreStage::Last, despawn.system());
    }
}

pub struct DespawnAfter(pub f32);

fn despawn(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut DespawnAfter)>) {
    for (entity, mut despawn_after) in query.iter_mut() {
        despawn_after.0 -= time.delta_seconds();

        if despawn_after.0 <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
