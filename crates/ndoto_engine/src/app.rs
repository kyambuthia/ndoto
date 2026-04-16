use bevy::{prelude::*, time::Fixed};

use crate::physics_backend::PhysicsBackendPlugin;

pub const SIMULATION_HZ: f64 = 60.0;

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimulationTick(pub u64);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum EngineFixedSet {
    Begin,
    Input,
    Simulation,
    End,
}

pub struct EngineCorePlugin;

impl Plugin for EngineCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(SIMULATION_HZ))
            .add_plugins(PhysicsBackendPlugin)
            .init_resource::<SimulationTick>()
            .configure_sets(
                FixedUpdate,
                (
                    EngineFixedSet::Begin,
                    EngineFixedSet::Input,
                    EngineFixedSet::Simulation,
                    EngineFixedSet::End,
                )
                    .chain(),
            )
            .add_systems(
                FixedUpdate,
                advance_simulation_tick.in_set(EngineFixedSet::Begin),
            );
    }
}

fn advance_simulation_tick(mut tick: ResMut<SimulationTick>) {
    tick.0 += 1;
}
