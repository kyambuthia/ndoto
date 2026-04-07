mod rendering;
mod time;

use bevy::prelude::*;
use rendering::DimensionalReadabilityPlugin;
use time::TimeReversalPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.055, 0.06, 0.08)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ndoto :: dimensional readability prototype".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((DimensionalReadabilityPlugin, TimeReversalPlugin))
        .run();
}
