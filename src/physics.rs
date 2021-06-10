use bevy::app::Events;
use bevy::ecs::query::WorldQuery;
use bevy::prelude::*;
use bevy_rapier3d::physics;
use bevy_rapier3d::physics::{
    JointsEntityMap, ModificationTracker, PhysicsHooksWithQueryObject, RapierConfiguration,
    SimulationToRenderTime,
};
use bevy_rapier3d::prelude::IntersectionEvent;
use bevy_rapier3d::rapier::dynamics::{CCDSolver, IntegrationParameters, IslandManager, JointSet};
use bevy_rapier3d::rapier::geometry::ContactEvent;
use bevy_rapier3d::rapier::geometry::{BroadPhase, NarrowPhase};
use bevy_rapier3d::rapier::pipeline::PhysicsPipeline;
use bevy_rapier3d::rapier::pipeline::QueryPipeline;
use std::marker::PhantomData;

/// A plugin responsible for setting up a full Rapier physics simulation pipeline and resources.
///
/// This will automatically setup all the resources needed to run a Rapier physics simulation including:
/// - The physics pipeline.
/// - The integration parameters.
/// - The rigid-body, collider, and joint, sets.
/// - The gravity.
/// - The broad phase and narrow-phase.
/// - The event queue.
/// - Systems responsible for executing one physics timestep at each Bevy update stage.
pub struct PhysicsPlugin<UserData>(PhantomData<UserData>);

impl<UserData> Default for PhysicsPlugin<UserData> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// The names of the default App stages
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum PhysicsStages {
    Creation,
    PostCreation,
    Update,
    PostUpdate,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PhysicsSystems {
    AttachBodiesAndColliders,
    CreateJoints,
    FinalizeColliderAttachToBodies,
    StepWorld,
    SyncTransforms,
}

impl<UserData: 'static + WorldQuery + Send + Sync> Plugin for PhysicsPlugin<UserData> {
    fn build(&self, app: &mut AppBuilder) {
        app.add_stage_before(
            CoreStage::PostUpdate,
            PhysicsStages::Creation,
            SystemStage::parallel(),
        )
        .add_stage_before(
            CoreStage::PostUpdate,
            PhysicsStages::PostCreation,
            SystemStage::parallel(),
        )
        .add_stage_before(
            CoreStage::PostUpdate,
            PhysicsStages::Update,
            SystemStage::parallel(),
        )
        .add_stage_before(
            CoreStage::PostUpdate,
            PhysicsStages::PostUpdate,
            SystemStage::parallel(),
        )
        .insert_resource(PhysicsPipeline::new())
        .insert_resource(QueryPipeline::new())
        .insert_resource(RapierConfiguration::default())
        .insert_resource(IntegrationParameters::default())
        .insert_resource(BroadPhase::new())
        .insert_resource(NarrowPhase::new())
        .insert_resource(IslandManager::new())
        .insert_resource(JointSet::new())
        .insert_resource(CCDSolver::new())
        .insert_resource(PhysicsHooksWithQueryObject::<UserData>(Box::new(())))
        .insert_resource(Events::<IntersectionEvent>::default())
        .insert_resource(Events::<ContactEvent>::default())
        .insert_resource(SimulationToRenderTime::default())
        .insert_resource(JointsEntityMap::default())
        .insert_resource(ModificationTracker::default())
        .add_system_to_stage(
            PhysicsStages::Creation,
            physics::attach_bodies_and_colliders_system
                .system()
                .label(PhysicsSystems::AttachBodiesAndColliders),
        )
        .add_system_to_stage(
            PhysicsStages::Creation,
            physics::create_joints_system
                .system()
                .label(PhysicsSystems::CreateJoints),
        )
        .add_system_to_stage(
            PhysicsStages::PostCreation,
            physics::finalize_collider_attach_to_bodies
                .system()
                .label(PhysicsSystems::FinalizeColliderAttachToBodies),
        )
        .add_system_to_stage(
            PhysicsStages::Update,
            physics::step_world_system::<UserData>
                .system()
                .label(PhysicsSystems::StepWorld),
        )
        .add_system_to_stage(
            PhysicsStages::PostUpdate,
            physics::sync_transforms
                .system()
                .label(PhysicsSystems::SyncTransforms),
        )
        .add_system_to_stage(CoreStage::Last, physics::collect_removals.system());
    }
}
