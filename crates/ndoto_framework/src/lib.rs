pub mod dimension;
pub mod input;
pub mod movement;
pub mod physics;

use bevy::prelude::*;

pub struct FrameworkPlugin;

impl Plugin for FrameworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            dimension::DimensionPlugin,
            input::PlayerInputPlugin,
            movement::MovementPlugin,
            physics::PhysicsPlugin,
        ));
    }
}
