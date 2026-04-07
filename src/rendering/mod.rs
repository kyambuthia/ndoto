pub mod camera;
pub mod scene;

use bevy::prelude::*;

use self::{
    camera::{
        RenderModeState, animate_view, setup_camera, update_atmosphere, update_render_mode,
        update_view_projection,
    },
    scene::{animate_dream_light, setup_scene, update_lighting},
};

pub struct DimensionalReadabilityPlugin;

impl Plugin for DimensionalReadabilityPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.72, 0.74, 0.8),
            brightness: 22.0,
            ..default()
        })
        .init_resource::<RenderModeState>()
        .add_systems(Startup, (setup_scene, setup_camera))
        .add_systems(
            Update,
            (
                update_render_mode,
                update_view_projection,
                update_atmosphere,
                animate_view,
                animate_dream_light,
                update_lighting,
            )
                .chain(),
        );
    }
}
