pub mod prototype;

use bevy::prelude::*;
use ndoto_engine::EngineCorePlugin;
use ndoto_framework::FrameworkPlugin;

pub struct NdotoGamePlugin;

impl Plugin for NdotoGamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.055, 0.06, 0.08)))
            .add_plugins((
                EngineCorePlugin,
                FrameworkPlugin,
                prototype::PrototypeGamePlugin,
            ));
    }
}
