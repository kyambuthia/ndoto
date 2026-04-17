use bevy::{
    prelude::*,
    render::{
        RenderPlugin,
        settings::{InstanceFlags, RenderCreation, WgpuSettings},
    },
};
use ndoto_game::NdotoGamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "ndoto :: dimensional readability prototype".into(),
                    resolution: (960, 540).into(),
                    ..default()
                }),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    // Validation is useful during engine work, but it makes debug runs noisier
                    // and adds overhead on already-constrained machines.
                    instance_flags: InstanceFlags::empty(),
                    ..default()
                }),
                ..default()
            }))
        .add_plugins(NdotoGamePlugin)
        .run();
}
