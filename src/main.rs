use bevy::prelude::*;
use ndoto_game::NdotoGamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ndoto :: dimensional readability prototype".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(NdotoGamePlugin)
        .run();
}
