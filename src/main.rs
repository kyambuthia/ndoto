mod rendering;

use bevy::prelude::*;
use rendering::RenderingSandboxPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.055, 0.06, 0.08)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ndoto :: multidimensional rendering sandbox".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RenderingSandboxPlugin)
        .run();
}
