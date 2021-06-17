use std::f32::consts::PI;

use bevy::{prelude::*, tasks::ComputeTaskPool};
use bevy_rapier3d::{na::UnitQuaternion, prelude::*};
use rand::Rng;

use crate::{despawn::DespawnAfter, random::Random, PhysicsFlags};

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(release_weapon_trigger.system().before("fire_weapon"))
            .add_system(fire_weapon.system().label("fire_weapon"))
            .add_system(cooldown_weapon.system().after("fire_weapon"))
            .add_system(reload_weapon.system().after("fire_weapon"));
    }
}

#[derive(Bundle)]
pub struct WeaponBundle {
    pub weapon: Weapon,
    pub fire_mode: WeaponFireMode,
    pub trigger: WeaponTrigger,
    pub ammo_count: WeaponAmmoCount,
    pub cooldown_time: WeaponCooldownTime,
    pub reload_time: WeaponReloadTime,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

pub struct Weapon {
    pub ammo_capacity: i32,
    pub rate_of_file: f32,
    pub reload_time: f32,
    pub projectile_shape: SharedShape,
    pub projectile_scene: Handle<Scene>,
    pub shoot_sound: Handle<AudioSource>,
    pub reload_sound: Handle<AudioSource>,
}

pub struct WeaponEnabled;

pub enum WeaponFireMode {
    Semi,
    Burst(u32),
    Auto,
}

pub struct WeaponTrigger {
    pub release_required: bool,
}

pub struct WeaponAmmoCount(pub i32);

pub struct WeaponCooldownTime(pub f32);

pub struct WeaponReloadTime(pub f32);

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
                    collision_groups: InteractionGroups::new(
                        PhysicsFlags::PROJECTILE.bits(),
                        !PhysicsFlags::PLAYER.bits(),
                    ),
                    ..Default::default()
                },
                ..Default::default()
            },
            rigid_body_position_sync: RigidBodyPositionSync::Discrete,
        }
    }
}

pub struct Projectile;

fn release_weapon_trigger(
    pool: Res<ComputeTaskPool>,
    input: Res<Input<MouseButton>>,
    mut query: Query<&mut WeaponTrigger>,
) {
    query.par_for_each_mut(&pool, 32, |mut trigger| {
        if input.just_released(MouseButton::Left) {
            trigger.release_required = false;
        }
    });
}

fn fire_weapon(
    mut commands: Commands,
    audio: Res<Audio>,
    input: Res<Input<MouseButton>>,
    mut random: ResMut<Random>,
    mut query: Query<
        (
            &Weapon,
            &WeaponFireMode,
            &mut WeaponTrigger,
            &mut WeaponAmmoCount,
            &mut WeaponReloadTime,
            &mut WeaponCooldownTime,
            &GlobalTransform,
        ),
        With<WeaponEnabled>,
    >,
) {
    if input.pressed(MouseButton::Left) {
        for (
            weapon,
            weapon_fire_mode,
            mut weapon_trigger,
            mut weapon_ammo_count,
            mut weapon_reload_time,
            mut weapon_cooldown_time,
            weapon_transform,
        ) in query.iter_mut()
        {
            if weapon_trigger.release_required
                || weapon_reload_time.0 > 0.0
                || weapon_cooldown_time.0 > 0.0
            {
                continue;
            }

            if weapon_ammo_count.0 <= 0 {
                weapon_reload_time.0 = weapon.reload_time;
                weapon_trigger.release_required = true;
                continue;
            }

            weapon_ammo_count.0 -= 1;

            if weapon_ammo_count.0 <= 0 {
                weapon_reload_time.0 = weapon.reload_time;
                weapon_trigger.release_required = true;
            }

            if let WeaponFireMode::Semi = *weapon_fire_mode {
                weapon_trigger.release_required = true;
            }

            weapon_cooldown_time.0 = 1.0 / weapon.rate_of_file;

            audio.play(weapon.shoot_sound.clone());

            let random_rotation = UnitQuaternion::from_euler_angles(
                PI * 1.5 / 180.0,
                0.0,
                random.generator.gen_range(-PI..=PI),
            );
            let mut projectile_bundle = ProjectileBundle::default();
            projectile_bundle.rigid_body.position.position = Isometry::from_parts(
                weapon_transform.translation.into(),
                UnitQuaternion::from(weapon_transform.rotation) * random_rotation,
            );
            projectile_bundle.rigid_body.velocity = RigidBodyVelocity {
                linvel: UnitQuaternion::from(weapon_transform.rotation)
                    * random_rotation
                    * Vector::z()
                    * 100.0,
                ..Default::default()
            };
            projectile_bundle.collider.shape = weapon.projectile_shape.clone();

            commands
                .spawn_bundle(projectile_bundle)
                .with_children(|parent| {
                    parent.spawn_scene(weapon.projectile_scene.clone());
                });
        }
    }
}

fn cooldown_weapon(
    time: Res<Time>,
    pool: Res<ComputeTaskPool>,
    mut query: Query<&mut WeaponCooldownTime>,
) {
    query.par_for_each_mut(&pool, 32, |mut cooldown_time| {
        if cooldown_time.0 > 0.0 {
            cooldown_time.0 -= time.delta_seconds()
        }
    });
}

fn reload_weapon(
    audio: Res<Audio>,
    time: Res<Time>,
    pool: Res<ComputeTaskPool>,
    mut query: Query<(&Weapon, &mut WeaponAmmoCount, &mut WeaponReloadTime)>,
) {
    query.par_for_each_mut(&pool, 32, |(weapon, mut ammo_count, mut reload_time)| {
        if reload_time.0 > 0.0 {
            reload_time.0 -= time.delta_seconds();

            if reload_time.0 <= 0.0 {
                ammo_count.0 = weapon.ammo_capacity;
                audio.play(weapon.reload_sound.clone());
            }
        }
    });
}
