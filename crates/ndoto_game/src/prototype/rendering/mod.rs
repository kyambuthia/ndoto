pub mod camera;
pub mod scene;

use bevy::prelude::*;
use ndoto_engine::EngineFixedSet;

use self::{
    camera::{
        animate_view, setup_camera, update_atmosphere, update_render_mode, update_view_projection,
    },
    scene::{animate_dream_light, setup_scene, update_lighting},
};

pub struct DimensionalReadabilityPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RenderingUpdateSet;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PrototypeSimulationSet;

impl Plugin for DimensionalReadabilityPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.72, 0.74, 0.8),
            brightness: 22.0,
            ..default()
        })
        .configure_sets(Update, RenderingUpdateSet)
        .configure_sets(FixedUpdate, PrototypeSimulationSet)
        .add_systems(Startup, (setup_scene, setup_camera))
        .add_systems(
            FixedUpdate,
            (update_render_mode, animate_dream_light)
                .chain()
                .in_set(EngineFixedSet::Simulation)
                .in_set(PrototypeSimulationSet),
        )
        .add_systems(
            Update,
            (
                update_view_projection,
                update_atmosphere,
                animate_view,
                update_lighting,
            )
                .chain()
                .in_set(RenderingUpdateSet),
        );
    }
}
