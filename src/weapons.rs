use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(CoreStage::Last, despawn_projectiles.system());
    }
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    pub lifespan: ProjectileLifespan,
    #[bundle]
    pub rigid_body: RigidBodyBundle,
    #[bundle]
    pub collider: ColliderBundle,
    pub rigid_body_position_sync: RigidBodyPositionSync,
}

impl Default for ProjectileBundle {
    fn default() -> Self {
        Self {
            lifespan: ProjectileLifespan(1.0),
            rigid_body: RigidBodyBundle {
                body_type: RigidBodyType::Dynamic,
                forces: RigidBodyForces {
                    gravity_scale: 0.0,
                    ..Default::default()
                },
                ccd: RigidBodyCcd {
                    ccd_thickness: 0.0,
                    ccd_max_dist: 0.2,
                    ccd_active: false,
                    ccd_enabled: true,
                },
                ..Default::default()
            },
            collider: ColliderBundle {
                collider_type: ColliderType::Sensor,
                ..Default::default()
            },
            rigid_body_position_sync: RigidBodyPositionSync::Discrete,
        }
    }
}

pub struct ProjectileLifespan(f32);

fn despawn_projectiles(
    mut commands: Commands,
    integration_parameters: Res<IntegrationParameters>,
    mut query: Query<(Entity, &mut ProjectileLifespan)>,
) {
    for (entity, mut lifespan) in query.iter_mut() {
        lifespan.0 -= integration_parameters.dt;

        if lifespan.0 <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
