use bevy::prelude::*;
use bevy_rapier3d::physics::{JointsEntityMap, ModificationTracker};
use bevy_rapier3d::prelude::PhysicsPipeline;
use bevy_rapier3d::rapier::dynamics::{CCDSolver, IslandManager, JointSet};
use bevy_rapier3d::rapier::geometry::{BroadPhase, NarrowPhase};
use bevy_rapier3d::rapier::pipeline::QueryPipeline;

use crate::AppState;

pub struct CleanupPlugin;

impl Plugin for CleanupPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(CleanupConfig {
            next_state_after_cleanup: None,
        })
        .add_system_to_stage(CoreStage::First, cleanup_entities.system())
        .add_system_to_stage(CoreStage::Last, cleanup_physics_states.system())
        .add_system_to_stage(CoreStage::Last, switch_state_after_cleanup.system());
    }
}

pub struct CleanupConfig {
    pub next_state_after_cleanup: Option<AppState>,
}

fn cleanup_entities(app_state: Res<State<AppState>>, mut commands: Commands, query: Query<Entity>) {
    if *app_state.current() != AppState::Cleanup {
        return;
    }

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn cleanup_physics_states(
    app_state: Res<State<AppState>>,
    mut physics_pipeline: ResMut<PhysicsPipeline>,
    mut query_pipeline: ResMut<QueryPipeline>,
    mut broad_phase: ResMut<BroadPhase>,
    mut narrow_phase: ResMut<NarrowPhase>,
    mut island_manager: ResMut<IslandManager>,
    mut joint_set: ResMut<JointSet>,
    mut ccd_solver: ResMut<CCDSolver>,
    mut joint_entity_map: ResMut<JointsEntityMap>,
    mut modification_tracker: ResMut<ModificationTracker>,
) {
    if *app_state.current() != AppState::Cleanup {
        return;
    }

    *physics_pipeline = Default::default();
    *query_pipeline = Default::default();
    *broad_phase = BroadPhase::new();
    *narrow_phase = NarrowPhase::new();
    *island_manager = IslandManager::new();
    *joint_set = JointSet::new();
    *ccd_solver = CCDSolver::new();
    *joint_entity_map = Default::default();
    *modification_tracker = Default::default();
}

fn switch_state_after_cleanup(
    cleanup_config: Res<CleanupConfig>,
    mut app_state: ResMut<State<AppState>>,
) {
    if *app_state.current() != AppState::Cleanup {
        return;
    }

    match cleanup_config.next_state_after_cleanup {
        None => panic!("CleanupConfig.next_state_after_cleanup is not specified!"),
        Some(AppState::Cleanup) => {
            panic!("CleanupConfig.next_state_after_cleanup must not be AppState::Cleanup")
        }
        Some(state) => app_state.set(state).unwrap(),
    }
}
