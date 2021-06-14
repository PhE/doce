use bevy::{pbr::AmbientLight, prelude::*, render::mesh::shape};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_rapier3d::prelude::*;

struct AppData {
    cube: Handle<Mesh>,
    cube_fragment: Handle<Mesh>,
    spawned: bool,
}

struct Despawn(f32);

struct Cube;

struct CubeFragment;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(EguiPlugin)
        .insert_resource(AppData {
            cube: Default::default(),
            cube_fragment: Default::default(),
            spawned: false,
        })
        .insert_resource(AmbientLight {
            brightness: 0.8,
            ..Default::default()
        })
        .add_startup_system(setup.system())
        .add_system(spawn.system())
        .add_system(handle_intersections.system())
        .add_system_to_stage(CoreStage::PostUpdate, debug.system())
        .add_system_to_stage(CoreStage::Last, despawn.system())
        .run();
}

fn setup(mut commands: Commands, mut data: ResMut<AppData>, mut meshes: ResMut<Assets<Mesh>>) {
    data.cube = meshes.add(shape::Box::new(0.5, 0.5, 0.5).into());
    data.cube_fragment = meshes.add(shape::Box::new(0.1, 0.1, 0.1).into());

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(shape::Box::new(0.5, 0.5, 2.0).into()),
            ..Default::default()
        })
        .insert_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Dynamic,
            position: vector!(0.0, 0.0, 1.0).into(),
            velocity: RigidBodyVelocity {
                angvel: vector!(0.0, 1000.0, 0.0),
                ..Default::default()
            },
            forces: RigidBodyForces {
                gravity_scale: 0.0,
                ..Default::default()
            },
            ccd: RigidBodyCcd {
                ccd_active: false,
                ccd_enabled: true,
                ccd_max_dist: 3.0,
                ccd_thickness: 0.0,
            },
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            collider_type: ColliderType::Sensor,
            shape: SharedShape::cuboid(0.5, 0.5, 2.0),
            flags: ColliderFlags {
                active_events: ActiveEvents::INTERSECTION_EVENTS,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(RigidBodyPositionSync::Discrete);
}

fn spawn(mut commands: Commands, mut data: ResMut<AppData>) {
    if !data.spawned {
        data.spawned = true;

        commands
            .spawn_bundle(PbrBundle {
                mesh: data.cube.clone(),
                ..Default::default()
            })
            .insert_bundle(RigidBodyBundle {
                body_type: RigidBodyType::KinematicPositionBased,
                ..Default::default()
            })
            .insert_bundle(ColliderBundle {
                shape: SharedShape::cuboid(0.5, 0.5, 0.5),
                ..Default::default()
            })
            .insert(RigidBodyPositionSync::Discrete)
            .insert(Cube);
    }
}

fn despawn(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Despawn)>) {
    for (entity, mut despawn) in query.iter_mut() {
        despawn.0 -= time.delta_seconds();

        if despawn.0 <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn handle_intersections(
    mut commands: Commands,
    mut data: ResMut<AppData>,
    mut intersection_events: EventReader<IntersectionEvent>,
    mut cube_query: Query<&mut ColliderFlags, With<Cube>>,
) {
    for intersection_event in intersection_events.iter() {
        if !intersection_event.intersecting {
            return;
        }

        let cube_entity: Entity;
        let mut cube_collider_flags: Mut<ColliderFlags>;

        if let Ok(flags) = cube_query.get_mut(intersection_event.collider1.entity()) {
            cube_entity = intersection_event.collider1.entity();
            cube_collider_flags = flags;
        } else if let Ok(flags) = cube_query.get_mut(intersection_event.collider2.entity()) {
            cube_entity = intersection_event.collider2.entity();
            cube_collider_flags = flags;
        } else {
            continue;
        }

        commands.entity(cube_entity).insert(Despawn(1.0));
        cube_collider_flags.collision_groups.filter = 0;
        data.spawned = false;

        for _ in 0..16 {
            commands
                .spawn_bundle(PbrBundle {
                    mesh: data.cube_fragment.clone(),
                    ..Default::default()
                })
                .insert_bundle(RigidBodyBundle {
                    body_type: RigidBodyType::Dynamic,
                    ..Default::default()
                })
                .insert_bundle(ColliderBundle {
                    shape: SharedShape::cuboid(0.1, 0.1, 0.1),
                    flags: ColliderFlags {
                        // collision_groups: InteractionGroups::none(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(RigidBodyPositionSync::Discrete)
                .insert(CubeFragment)
                .insert(Despawn(1.0));
        }
    }
}

fn debug(
    context: Res<EguiContext>,
    cube_query: Query<Entity, With<Cube>>,
    cube_fragment_query: Query<Entity, With<CubeFragment>>,
) {
    egui::Window::new("Count").show(context.ctx(), |ui| {
        let count = cube_query.iter().count();
        ui.label(format!("Cubes: {}", count));

        let count = cube_fragment_query.iter().count();
        ui.label(format!("Cube fragments: {}", count));
    });
}
