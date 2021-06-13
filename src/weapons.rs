use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::despawn::DespawnAfter;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut AppBuilder) {}
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    pub projectile: Projectile,
    pub despawn_after: DespawnAfter,
    #[bundle]
    pub rigid_body: RigidBodyBundle,
    #[bundle]
    pub collider: ColliderBundle,
    pub rigid_body_position_sync: RigidBodyPositionSync,
}

impl Default for ProjectileBundle {
    fn default() -> Self {
        Self {
            projectile: Projectile,
            despawn_after: DespawnAfter(1.0),
            rigid_body: RigidBodyBundle {
                body_type: RigidBodyType::Dynamic,
                forces: RigidBodyForces {
                    gravity_scale: 0.0,
                    ..Default::default()
                },
                ccd: RigidBodyCcd {
                    ccd_thickness: 0.0,
                    ccd_max_dist: 0.4,
                    ccd_active: false,
                    ccd_enabled: true,
                },
                ..Default::default()
            },
            collider: ColliderBundle {
                collider_type: ColliderType::Sensor,
                material: ColliderMaterial {
                    friction: 0.2,
                    restitution: 0.8,
                    ..Default::default()
                },
                flags: ColliderFlags {
                    active_events: ActiveEvents::INTERSECTION_EVENTS,
                    collision_groups: InteractionGroups::new(1 << 2, u32::MAX),
                    ..Default::default()
                },
                ..Default::default()
            },
            rigid_body_position_sync: RigidBodyPositionSync::Discrete,
        }
    }
}

pub struct Projectile;
