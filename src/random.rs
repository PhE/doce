use bevy::prelude::*;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;

use crate::Tick;

pub struct RandomPlugin;

impl Plugin for RandomPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Random {
            generator: Pcg64Mcg::seed_from_u64(0),
        })
        .add_system_to_stage(CoreStage::First, reseed_generator.system());
    }
}

pub struct Random {
    pub generator: Pcg64Mcg,
}

fn reseed_generator(tick: Res<Tick>, mut random: ResMut<Random>) {
    random.generator = Pcg64Mcg::seed_from_u64(tick.0 as u64);
}
