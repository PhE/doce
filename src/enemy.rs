use std::f32::consts::PI;

use bevy::{core::FixedTimestep, math::Vec3Swizzles, prelude::*, render::mesh::shape};
use bevy_rapier3d::{na::distance, prelude::*};
use rand::Rng;

use crate::{despawn::DespawnAfter, weapons::Projectile, AppState, Health, MainCharacter, Random};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init_enemy_resources.system())
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(enemy_movement.system())
                    .with_system(enemy_hit.system()),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_run_criteria(FixedTimestep::step(0.5))
                    .with_system(enemy_spawn.system()),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_run_criteria(FixedTimestep::step(1.0))
                    .with_system(enemy_director.system()),
            );
    }
}

#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub behavior: EnemyBehavior,
    pub health: Health,
    #[bundle]
    pub rigid_body: RigidBodyBundle,
    #[bundle]
    pub collider: ColliderBundle,
    pub rigid_body_position_sync: RigidBodyPositionSync,
}

#[derive(Bundle)]
pub struct EnemyBloodSplatterBundle {
    pub despawn_after: DespawnAfter,
    #[bundle]
    pub pbr: PbrBundle,
    #[bundle]
    pub rigid_body: RigidBodyBundle,
    #[bundle]
    pub collider: ColliderBundle,
    pub rigid_body_position_sync: RigidBodyPositionSync,
}

pub struct Enemy;

#[derive(Debug)]
pub enum EnemyBehavior {
    Idle,
    Wander(Vec3),
    Attack(Entity),
    Death,
}

struct EnemyResources {
    enemy_mesh: Handle<Mesh>,
    enemy_material: Handle<StandardMaterial>,
    enemy_shape: SharedShape,
    blood_mesh: Handle<Mesh>,
    blood_material: Handle<StandardMaterial>,
    blood_shape: SharedShape,
}

fn init_enemy_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(EnemyResources {
        enemy_mesh: meshes.add(Mesh::from(shape::Capsule {
            radius: 0.5,
            depth: 1.0,
            ..Default::default()
        })),
        enemy_material: materials.add(Color::DARK_GRAY.into()),
        enemy_shape: SharedShape::capsule(point!(0.0, 0.5, 0.0), point!(0.0, 1.5, 0.0), 0.5),
        blood_mesh: meshes.add(Mesh::from(shape::Box::new(0.1, 0.1, 0.1))),
        blood_material: materials.add(Color::RED.into()),
        blood_shape: SharedShape::cuboid(0.1, 0.1, 0.1),
    });
}

fn enemy_hit(
    mut commands: Commands,
    enemy_resources: Res<EnemyResources>,
    mut random: ResMut<Random>,
    mut intersection_events: EventReader<IntersectionEvent>,
    mut query_set: QuerySet<(
        Query<(&Transform, &RigidBodyVelocity), With<Projectile>>,
        Query<(
            &mut EnemyBehavior,
            &mut Health,
            &mut RigidBodyVelocity,
            &mut RigidBodyMassProps,
        )>,
    )>,
) {
    for intersection_event in intersection_events.iter() {
        if !intersection_event.intersecting {
            continue;
        }

        let collider1 = intersection_event.collider1;
        let collider2 = intersection_event.collider2;

        let projectile_entity: Entity;
        let projectile_transform: Transform;
        let projectile_velocity: RigidBodyVelocity;
        let enemy_entity: Entity;
        let mut enemy_behavior: Mut<EnemyBehavior>;
        let mut enemy_health: Mut<Health>;
        let mut enemy_velocity: Mut<RigidBodyVelocity>;
        let mut enemy_mass_props: Mut<RigidBodyMassProps>;

        let projectile_query = query_set.q0();

        if let Ok((&transform, &velocity)) = projectile_query.get(collider1.entity()) {
            projectile_entity = collider1.entity();
            projectile_transform = transform;
            projectile_velocity = velocity;
        } else if let Ok((&transform, &velocity)) = projectile_query.get(collider2.entity()) {
            projectile_entity = collider2.entity();
            projectile_transform = transform;
            projectile_velocity = velocity;
        } else {
            continue;
        }

        let enemy_query = query_set.q1_mut();

        if let Ok((behavior, health, velocity, mass_props)) =
            enemy_query.get_mut(collider1.entity())
        {
            enemy_entity = collider1.entity();
            enemy_behavior = behavior;
            enemy_health = health;
            enemy_velocity = velocity;
            enemy_mass_props = mass_props;
        } else if let Ok((behavior, health, velocity, mass_props)) =
            enemy_query.get_mut(collider2.entity())
        {
            enemy_entity = collider2.entity();
            enemy_behavior = behavior;
            enemy_health = health;
            enemy_velocity = velocity;
            enemy_mass_props = mass_props;
        } else {
            continue;
        }

        if let EnemyBehavior::Death = *enemy_behavior {
            continue;
        }

        commands.entity(projectile_entity).despawn_recursive();

        spawn_blood_splatters(
            &mut commands,
            projectile_transform.translation,
            &enemy_resources,
            &mut random,
        );

        enemy_health.0 -= 50.0;

        if enemy_health.0 <= 0.0 {
            commands.entity(enemy_entity).insert(DespawnAfter(4.0));
            *enemy_behavior = EnemyBehavior::Death;
            enemy_mass_props.flags = RigidBodyMassPropsFlags::empty();
            enemy_velocity.apply_impulse_at_point(
                &enemy_mass_props,
                projectile_velocity.linvel.normalize() * 10.0,
                projectile_transform.translation.into(),
            );
        }
    }
}

fn enemy_spawn(
    mut commands: Commands,
    resources: Res<EnemyResources>,
    mut random: ResMut<Random>,
    query: Query<Entity, With<Enemy>>,
) {
    let count = query.iter().count();

    if count < 100 {
        let mut position = Vec3::new(random.generator.gen_range(-24.5..24.5), 0.0, 24.5);

        if random.generator.gen_bool(0.5) {
            position.z = -position.z;
        }

        if random.generator.gen_bool(0.5) {
            position = position.zyx();
        }

        commands
            .spawn_bundle(EnemyBundle {
                enemy: Enemy,
                behavior: EnemyBehavior::Idle,
                health: Health(100.0),
                rigid_body: RigidBodyBundle {
                    body_type: RigidBodyType::Dynamic,
                    mass_properties: RigidBodyMassPropsFlags::ROTATION_LOCKED.into(),
                    position: position.into(),
                    ..Default::default()
                },
                collider: ColliderBundle {
                    shape: resources.enemy_shape.clone(),
                    flags: ColliderFlags {
                        collision_groups: InteractionGroups::new(1 << 1, u32::MAX),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                rigid_body_position_sync: RigidBodyPositionSync::Discrete,
            })
            .with_children(|parent| {
                parent.spawn_bundle(PbrBundle {
                    mesh: resources.enemy_mesh.clone(),
                    material: resources.enemy_material.clone(),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    ..Default::default()
                });
            });
    }
}

fn enemy_director(
    mut random: ResMut<Random>,
    character_query: Query<(Entity, &Transform), With<MainCharacter>>,
    mut enemy_query: Query<(&mut EnemyBehavior, &RigidBodyPosition)>,
) {
    for (mut behavior, position) in enemy_query.iter_mut() {
        if let EnemyBehavior::Death = *behavior {
            continue;
        }

        if let (Some(character), Some(distance)) =
            character_query
                .iter()
                .fold((None, None), |accumulator, (entity, transform)| {
                    let distance = distance(
                        &position.position.translation.vector.into(),
                        &transform.translation.into(),
                    );

                    match accumulator {
                        (None, _) => (Some(entity), Some(distance)),
                        (Some(_), Some(max_distance)) if max_distance < distance => {
                            (Some(entity), Some(distance))
                        }
                        _ => accumulator,
                    }
                })
        {
            if distance < 10.0 {
                *behavior = EnemyBehavior::Attack(character);
                continue;
            }
        }

        let direction = loop {
            let target_position = Vec3::new(
                random.generator.gen_range(-24.5..24.5),
                0.0,
                random.generator.gen_range(-24.5..24.5),
            );

            if let Some(direction) =
                (target_position - position.position.translation.into()).try_normalize()
            {
                break direction;
            }
        };

        *behavior = EnemyBehavior::Wander(direction);
    }
}

fn enemy_movement(
    character_query: Query<&Transform, With<MainCharacter>>,
    mut enemy_query: Query<(&EnemyBehavior, &RigidBodyPosition, &mut RigidBodyVelocity)>,
) {
    for (behavior, position, mut velocity) in enemy_query.iter_mut() {
        match behavior {
            EnemyBehavior::Wander(direction) => velocity.linvel = (*direction * 0.5).into(),
            EnemyBehavior::Attack(character) => {
                let character_transform = character_query.get(*character).unwrap();
                let direction = (character_transform.translation
                    - position.position.translation.into())
                .normalize();
                velocity.linvel = (direction * 1.0).into();
            }
            _ => (),
        }
    }
}

fn spawn_blood_splatters(
    commands: &mut Commands,
    position: Vec3,
    enemy_resources: &Res<EnemyResources>,
    random: &mut ResMut<Random>,
) {
    for _ in 0..16 {
        let direction = Quat::from_rotation_x(random.generator.gen_range(-PI..PI))
            * Quat::from_rotation_y(random.generator.gen_range(-PI..PI))
            * Vec3::Z;

        if direction.is_nan() {
            error!("NaN detected! {}", direction);
            continue;
        }

        commands.spawn_bundle(EnemyBloodSplatterBundle {
            despawn_after: DespawnAfter(1.0),
            pbr: PbrBundle {
                mesh: enemy_resources.blood_mesh.clone(),
                material: enemy_resources.blood_material.clone(),
                ..Default::default()
            },
            rigid_body: RigidBodyBundle {
                body_type: RigidBodyType::Dynamic,
                position: position.into(),
                velocity: RigidBodyVelocity {
                    linvel: (direction * 5.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            collider: ColliderBundle {
                shape: enemy_resources.blood_shape.clone(),
                flags: ColliderFlags {
                    collision_groups: InteractionGroups::new(1 << 3, !(1 << 2)),
                    ..Default::default()
                },
                ..Default::default()
            },
            rigid_body_position_sync: RigidBodyPositionSync::Discrete,
        });
    }
}
