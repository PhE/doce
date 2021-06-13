use bevy::{core::FixedTimestep, math::Vec3Swizzles, prelude::*, render::mesh::shape};
use bevy_rapier3d::{na::distance, prelude::*};
use rand::Rng;

use crate::{weapons::Projectile, AppState, Health, MainCharacter, Random};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init_enemy_resources.system())
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(enemy_movement.system())
                    .with_system(enemy_despawn.system())
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

pub struct Enemy;

#[derive(Debug)]
pub enum EnemyBehavior {
    Idle,
    Wander(Vec3),
    Attack(Entity),
    Death(f32),
}

struct EnemyResources {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    shape: SharedShape,
}

fn init_enemy_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(EnemyResources {
        mesh: meshes.add(Mesh::from(shape::Capsule {
            radius: 0.5,
            depth: 1.0,
            ..Default::default()
        })),
        material: materials.add(Color::RED.into()),
        shape: SharedShape::capsule(point!(0.0, 0.5, 0.0), point!(0.0, 1.5, 0.0), 0.5),
    });
}

fn enemy_hit(
    mut commands: Commands,
    mut intersection_events: EventReader<IntersectionEvent>,
    mut query_set: QuerySet<(
        Query<(Entity, &Transform, &RigidBodyVelocity), With<Projectile>>,
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
        let mut enemy_behavior: Mut<EnemyBehavior>;
        let mut enemy_health: Mut<Health>;
        let mut enemy_velocity: Mut<RigidBodyVelocity>;
        let mut enemy_mass_props: Mut<RigidBodyMassProps>;

        let projectile_query = query_set.q0();

        if let Ok((entity, &transform, &velocity)) = projectile_query.get(collider1.entity()) {
            projectile_entity = entity;
            projectile_transform = transform;
            projectile_velocity = velocity;
        } else if let Ok((entity, &transform, &velocity)) = projectile_query.get(collider2.entity())
        {
            projectile_entity = entity;
            projectile_transform = transform;
            projectile_velocity = velocity;
        } else {
            continue;
        }

        let enemy_query = query_set.q1_mut();

        if let Ok((behavior, health, velocity, mass_props)) =
            enemy_query.get_mut(collider1.entity())
        {
            enemy_behavior = behavior;
            enemy_health = health;
            enemy_velocity = velocity;
            enemy_mass_props = mass_props;
        } else if let Ok((behavior, health, velocity, mass_props)) =
            enemy_query.get_mut(collider2.entity())
        {
            enemy_behavior = behavior;
            enemy_health = health;
            enemy_velocity = velocity;
            enemy_mass_props = mass_props;
        } else {
            continue;
        }

        commands.entity(projectile_entity).despawn_recursive();

        enemy_health.0 -= 50.0;

        if enemy_health.0 <= 0.0 {
            *enemy_behavior = EnemyBehavior::Death(5.0);
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
                    shape: resources.shape.clone(),
                    ..Default::default()
                },
                rigid_body_position_sync: RigidBodyPositionSync::Discrete,
            })
            .with_children(|parent| {
                parent.spawn_bundle(PbrBundle {
                    mesh: resources.mesh.clone(),
                    material: resources.material.clone(),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    ..Default::default()
                });
            });
    }
}

fn enemy_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut EnemyBehavior)>,
) {
    for (entity, mut behavior) in query.iter_mut() {
        if let EnemyBehavior::Death(mut despawn_time) = *behavior {
            despawn_time -= time.delta_seconds();

            if despawn_time <= 0.0 {
                commands.entity(entity).despawn_recursive();
            } else {
                *behavior = EnemyBehavior::Death(despawn_time);
            }
        }
    }
}

fn enemy_director(
    mut random: ResMut<Random>,
    character_query: Query<(Entity, &Transform), With<MainCharacter>>,
    mut enemy_query: Query<(&mut EnemyBehavior, &RigidBodyPosition)>,
) {
    for (mut behavior, position) in enemy_query.iter_mut() {
        if let EnemyBehavior::Death(_) = *behavior {
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

        let target_position = Vec3::new(
            random.generator.gen_range(-24.5..24.5),
            0.0,
            random.generator.gen_range(-24.5..24.5),
        );
        let direction = (target_position - position.position.translation.into()).normalize();

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
