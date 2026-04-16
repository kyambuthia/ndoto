use bevy::{camera::ScalingMode, prelude::*};
use ndoto_framework::{
    dimension::{DimensionState, SpatialMode},
    input::FixedPlayerInput,
    movement::PlayerControlled,
};

use crate::prototype::rendering::scene::SceneRoot;

const ATMOSPHERE_LERP_SPEED: f32 = 2.8;
const CAMERA_LERP_SPEED: f32 = 6.5;
const ROOT_LERP_SPEED: f32 = 5.0;
const TWO_D_DEPTH_SCALE: f32 = 0.12;
const ONE_D_HEIGHT_SCALE: f32 = 0.08;
const ONE_D_DEPTH_SCALE: f32 = 0.02;

#[derive(Component)]
pub struct SandboxCamera;

pub fn setup_camera(mut commands: Commands) {
    let spec = render_mode_spec(SpatialMode::ThreeD);

    commands.spawn((
        Name::new("SandboxCamera"),
        SandboxCamera,
        Camera3d::default(),
        spec.projection,
        DistanceFog {
            color: Color::srgb(0.11, 0.115, 0.135),
            falloff: FogFalloff::Linear {
                start: 16.0,
                end: 34.0,
            },
            ..default()
        },
        Transform::from_translation(spec.eye).looking_at(spec.focus, Vec3::Y),
    ));
}

pub fn update_render_mode(
    player_input: Res<FixedPlayerInput>,
    mut dimension_state: ResMut<DimensionState>,
) {
    if player_input.switch_to_3d {
        dimension_state.set_spatial_mode(SpatialMode::ThreeD);
    } else if player_input.switch_to_2d {
        dimension_state.set_spatial_mode(SpatialMode::TwoD);
    } else if player_input.switch_to_1d {
        dimension_state.set_spatial_mode(SpatialMode::OneD);
    }
}

pub fn update_view_projection(
    dimension_state: Res<DimensionState>,
    mut camera: Single<&mut Projection, With<SandboxCamera>>,
) {
    **camera = render_mode_spec(dimension_state.spatial_mode).projection;
}

pub fn update_atmosphere(
    time: Res<Time>,
    dimension_state: Res<DimensionState>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    mut fog: Single<&mut DistanceFog, With<SandboxCamera>>,
) {
    let spec = active_render_mode_spec(&dimension_state);
    let blend = smoothing_factor(ATMOSPHERE_LERP_SPEED, time.delta_secs());

    clear_color.0 = clear_color.0.mix(&spec.clear_color, blend);
    ambient_light.color = ambient_light.color.mix(&spec.ambient_color, blend);
    ambient_light.brightness += (spec.ambient_brightness - ambient_light.brightness) * blend;

    fog.color = fog.color.mix(&spec.fog_color, blend);
    if let FogFalloff::Linear { start, end } = &mut fog.falloff {
        *start += (spec.fog_start - *start) * blend;
        *end += (spec.fog_end - *end) * blend;
    }
}

pub fn animate_view(
    time: Res<Time>,
    dimension_state: Res<DimensionState>,
    mut camera: Single<&mut Transform, (With<SandboxCamera>, Without<SceneRoot>)>,
    mut scene_root: Single<&mut Transform, (With<SceneRoot>, Without<SandboxCamera>)>,
    player: Single<&Transform, (With<PlayerControlled>, Without<SandboxCamera>)>,
) {
    let spec = render_mode_spec(dimension_state.spatial_mode);
    let follow_focus = follow_focus(dimension_state.spatial_mode, player.translation);
    let desired_camera = Transform::from_translation(follow_focus + (spec.eye - spec.focus))
        .looking_at(follow_focus, Vec3::Y);

    let camera_blend = smoothing_factor(CAMERA_LERP_SPEED, time.delta_secs());
    camera.translation = camera
        .translation
        .lerp(desired_camera.translation, camera_blend);
    camera.rotation = camera.rotation.slerp(desired_camera.rotation, camera_blend);

    let root_blend = smoothing_factor(ROOT_LERP_SPEED, time.delta_secs());
    scene_root.scale = scene_root.scale.lerp(spec.scene_scale, root_blend);
}

pub(crate) fn active_render_mode_spec(dimension_state: &DimensionState) -> RenderModeSpec {
    let mut spec = render_mode_spec(dimension_state.spatial_mode);
    if dimension_state.is_four_d() {
        apply_four_d_overlay(&mut spec);
    }
    spec
}

pub(crate) fn render_mode_spec(mode: SpatialMode) -> RenderModeSpec {
    match mode {
        SpatialMode::ThreeD => RenderModeSpec {
            eye: Vec3::new(11.5, 8.5, 13.0),
            focus: Vec3::new(0.0, 1.6, 0.0),
            scene_scale: Vec3::ONE,
            clear_color: Color::srgb(0.055, 0.06, 0.08),
            ambient_color: Color::srgb(0.72, 0.74, 0.8),
            ambient_brightness: 22.0,
            fog_color: Color::srgb(0.11, 0.115, 0.135),
            fog_start: 16.0,
            fog_end: 34.0,
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 50.0_f32.to_radians(),
                ..default()
            }),
            point_light_range: 24.0,
        },
        SpatialMode::TwoD => RenderModeSpec {
            eye: Vec3::new(0.0, 8.0, 18.0),
            focus: Vec3::new(0.0, 1.5, 0.0),
            scene_scale: Vec3::new(1.0, 1.0, TWO_D_DEPTH_SCALE),
            clear_color: Color::srgb(0.07, 0.076, 0.09),
            ambient_color: Color::srgb(0.75, 0.76, 0.8),
            ambient_brightness: 19.5,
            fog_color: Color::srgb(0.13, 0.135, 0.155),
            fog_start: 11.0,
            fog_end: 24.0,
            projection: Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: 10.5,
                },
                ..OrthographicProjection::default_3d()
            }),
            point_light_range: 3.0,
        },
        SpatialMode::OneD => RenderModeSpec {
            eye: Vec3::new(0.0, 2.3, 20.0),
            focus: Vec3::new(0.0, 2.3, 0.0),
            scene_scale: Vec3::new(1.0, ONE_D_HEIGHT_SCALE, ONE_D_DEPTH_SCALE),
            clear_color: Color::srgb(0.082, 0.088, 0.102),
            ambient_color: Color::srgb(0.78, 0.79, 0.82),
            ambient_brightness: 16.0,
            fog_color: Color::srgb(0.16, 0.165, 0.185),
            fog_start: 8.0,
            fog_end: 16.0,
            projection: Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: 4.2,
                },
                ..OrthographicProjection::default_3d()
            }),
            point_light_range: 1.0,
        },
    }
}

fn apply_four_d_overlay(spec: &mut RenderModeSpec) {
    spec.clear_color = spec
        .clear_color
        .mix(&Color::srgb(0.028, 0.038, 0.062), 0.55);
    spec.ambient_color = spec.ambient_color.mix(&Color::srgb(0.56, 0.66, 0.78), 0.4);
    spec.ambient_brightness *= 0.9;
    spec.fog_color = spec.fog_color.mix(&Color::srgb(0.08, 0.12, 0.18), 0.5);
    spec.fog_start *= 0.82;
    spec.fog_end *= 0.9;
    spec.point_light_range *= 1.25;
}

pub(crate) fn smoothing_factor(speed: f32, delta_seconds: f32) -> f32 {
    1.0 - (-speed * delta_seconds).exp()
}

fn follow_focus(spatial_mode: SpatialMode, player_translation: Vec3) -> Vec3 {
    match spatial_mode {
        SpatialMode::ThreeD => Vec3::new(player_translation.x, 1.6, player_translation.z),
        SpatialMode::TwoD => Vec3::new(player_translation.x, 1.5, 0.0),
        SpatialMode::OneD => Vec3::new(player_translation.x, 2.3, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::Projection;

    #[test]
    fn render_mode_spec_3d_has_perspective_projection() {
        let spec = render_mode_spec(SpatialMode::ThreeD);
        assert!(matches!(spec.projection, Projection::Perspective(_)));
        assert_eq!(spec.scene_scale, Vec3::ONE);
    }

    #[test]
    fn render_mode_spec_2d_has_orthographic_projection() {
        let spec = render_mode_spec(SpatialMode::TwoD);
        assert!(matches!(spec.projection, Projection::Orthographic(_)));
        assert_eq!(spec.scene_scale.z, TWO_D_DEPTH_SCALE);
    }

    #[test]
    fn render_mode_spec_1d_has_orthographic_projection() {
        let spec = render_mode_spec(SpatialMode::OneD);
        assert!(matches!(spec.projection, Projection::Orthographic(_)));
        assert_eq!(spec.scene_scale.y, ONE_D_HEIGHT_SCALE);
        assert_eq!(spec.scene_scale.z, ONE_D_DEPTH_SCALE);
    }

    #[test]
    fn render_mode_spec_1d_is_more_compressed_than_2d() {
        let spec_2d = render_mode_spec(SpatialMode::TwoD);
        let spec_1d = render_mode_spec(SpatialMode::OneD);
        assert!(spec_1d.scene_scale.y < spec_2d.scene_scale.y);
        assert!(spec_1d.scene_scale.z < spec_2d.scene_scale.z);
    }

    #[test]
    fn smoothing_factor_returns_zero_at_zero_speed() {
        let result = smoothing_factor(0.0, 1.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn smoothing_factor_returns_zero_at_zero_delta() {
        let result = smoothing_factor(1.0, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn smoothing_factor_approaches_one_for_large_speed() {
        let result = smoothing_factor(100.0, 1.0);
        assert!((result - 1.0).abs() < 0.001);
    }

    #[test]
    fn active_render_mode_spec_adds_four_d_grade() {
        let mut state = DimensionState::default();
        state.toggle_four_d();

        let four_d = active_render_mode_spec(&state);
        let three_d = render_mode_spec(SpatialMode::ThreeD);
        assert_ne!(four_d.clear_color, three_d.clear_color);
        assert!(four_d.fog_start < three_d.fog_start);
    }

    #[test]
    fn follow_focus_constrains_non_3d_modes() {
        assert_eq!(
            follow_focus(SpatialMode::ThreeD, Vec3::new(3.0, 9.0, 5.0)),
            Vec3::new(3.0, 1.6, 5.0)
        );
        assert_eq!(
            follow_focus(SpatialMode::TwoD, Vec3::new(3.0, 9.0, 5.0)),
            Vec3::new(3.0, 1.5, 0.0)
        );
        assert_eq!(
            follow_focus(SpatialMode::OneD, Vec3::new(3.0, 9.0, 5.0)),
            Vec3::new(3.0, 2.3, 0.0)
        );
    }
}

pub(crate) struct RenderModeSpec {
    pub(crate) eye: Vec3,
    pub(crate) focus: Vec3,
    pub(crate) scene_scale: Vec3,
    pub(crate) clear_color: Color,
    pub(crate) ambient_color: Color,
    pub(crate) ambient_brightness: f32,
    pub(crate) fog_color: Color,
    pub(crate) fog_start: f32,
    pub(crate) fog_end: f32,
    pub(crate) projection: Projection,
    pub(crate) point_light_range: f32,
}
