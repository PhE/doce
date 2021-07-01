use bevy::prelude::*;

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(CoreStage::Last, despawn.system());
    }
}

pub struct DespawnAfter(pub f32);

fn despawn(mut commands: Commands, time: Res<Time>, query: Query<(Entity, &mut DespawnAfter)>) {
    query.for_each_mut(|(entity, mut despawn_after)| {
        despawn_after.0 -= time.delta_seconds();

        if despawn_after.0 <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    });
}
