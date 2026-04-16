pub mod rendering;
pub mod time;

use bevy::prelude::*;

pub struct PrototypeGamePlugin;

impl Plugin for PrototypeGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            rendering::DimensionalReadabilityPlugin,
            time::TimeReversalPlugin,
        ));
    }
}
