use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::io::prelude::*;

use crate::AppState;
use crate::GameReplay;
use crate::MainCharacter;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<DebugSimulationStateEvent>()
            .add_system_to_stage(CoreStage::Last, debug_simulation_state.system());

        let labels: Vec<_> = app
            .app
            .schedule
            .iter_stages()
            .map(|(label, _)| format!("{:?}", label))
            .collect();
        info!("Stages execution order:\n\t{}", labels.join("\n\t"));
    }
}

pub enum DebugSimulationStateEvent {
    Record,
    Compare,
}

pub struct DebugRigidBodyIndex(pub usize);

#[allow(unreachable_code, unused_mut, unused_variables)]
fn debug_simulation_state(
    mut debug_simulation_state_events: EventReader<DebugSimulationStateEvent>,
    mut game_replay: ResMut<GameReplay>,
    query: Query<&Transform, With<MainCharacter>>,
    rigid_body_query: Query<(&RigidBodyPosition, &DebugRigidBodyIndex)>,
    mut success_count: Local<i32>,
    mut fail_count: Local<i32>,
    mut app_state: ResMut<State<AppState>>,
) {
    for event in debug_simulation_state_events.iter() {
        let transform = query.single().unwrap();

        match event {
            DebugSimulationStateEvent::Record => {
                game_replay.main_character_final_position = transform.translation;
            }
            DebugSimulationStateEvent::Compare => {
                info!(
                    "Comparing main character position at the end of replay:
\tIdentical: {}
\tRecorded position: {}. Binary: [{:032b}, {:032b}, {:032b}]
\tActual   position: {}. Binary: [{:032b}, {:032b}, {:032b}]
\tDifference: {}",
                    game_replay.main_character_final_position == transform.translation,
                    game_replay.main_character_final_position,
                    game_replay.main_character_final_position.x.to_bits(),
                    game_replay.main_character_final_position.y.to_bits(),
                    game_replay.main_character_final_position.z.to_bits(),
                    transform.translation,
                    transform.translation.x.to_bits(),
                    transform.translation.y.to_bits(),
                    transform.translation.z.to_bits(),
                    game_replay.main_character_final_position - transform.translation
                );

                return;

                let mut collection: Vec<_> = rigid_body_query.iter().collect();
                collection.sort_by(|(_, fp_1), (_, fp_2)| fp_1.0.cmp(&fp_2.0));

                if std::path::Path::new("final_state.txt").is_file() {
                    let file = std::fs::File::open("final_state.txt").unwrap();
                    let buffer = std::io::BufReader::new(file);
                    let mut lines_iter = buffer.lines().map(|l| l.unwrap());
                    let mut identical_counts = 0;
                    let mut total_counts = 0;

                    for (position, final_position) in collection {
                        total_counts += 1;

                        let fp: Vec3 = position.position.translation.into();
                        let line = format!("{},{},{},{}", final_position.0, fp.x, fp.y, fp.z);
                        let file_line = lines_iter.next().unwrap();

                        if line == file_line {
                            identical_counts += 1;
                            // } else {
                            //     info!("Rigid body mismatch:\n\t{}\n\t{}",line,file_line);
                        }
                    }

                    if identical_counts == total_counts {
                        *success_count += 1;
                    } else {
                        *fail_count += 1;
                    }

                    info!(
                        "Success count: {}. Fail count: {}",
                        *success_count, *fail_count
                    );
                    // info!("Final rigid body position summary:\n\tIdentital counts: {}\n\tTotal counts: {}", identical_counts, total_counts);
                } else {
                    let mut file = std::fs::File::create("final_state.txt").unwrap();

                    for (position, final_position) in collection {
                        let fp: Vec3 = position.position.translation.into();
                        write!(
                            &mut file,
                            "{},{},{},{}\n",
                            final_position.0, fp.x, fp.y, fp.z
                        )
                        .unwrap();
                    }
                }
            }
        }
    }
}
