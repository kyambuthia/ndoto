use bevy::{camera::ScalingMode, prelude::*};

use crate::rendering::scene::SceneRoot;

const ATMOSPHERE_LERP_SPEED: f32 = 2.8;
const CAMERA_LERP_SPEED: f32 = 6.5;
const ROOT_LERP_SPEED: f32 = 5.0;
const TWO_D_DEPTH_SCALE: f32 = 0.12;
const ONE_D_HEIGHT_SCALE: f32 = 0.08;
const ONE_D_DEPTH_SCALE: f32 = 0.02;

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RenderMode {
    #[default]
    ThreeD,
    TwoD,
    OneD,
    FourD,
}

#[derive(Resource, Clone, Debug)]
pub struct RenderModeState {
    pub mode: RenderMode,
    pub spatial_mode: RenderMode,
}

impl Default for RenderModeState {
    fn default() -> Self {
        Self {
            mode: RenderMode::ThreeD,
            spatial_mode: RenderMode::ThreeD,
        }
    }
}

impl RenderModeState {
    pub fn is_four_d(&self) -> bool {
        self.mode == RenderMode::FourD
    }

    pub fn effective_spatial_mode(&self) -> RenderMode {
        match self.mode {
            RenderMode::FourD => self.spatial_mode,
            spatial_mode => spatial_mode,
        }
    }

    pub fn set_spatial_mode(&mut self, mode: RenderMode) {
        debug_assert!(mode != RenderMode::FourD);
        self.spatial_mode = mode;
        if !self.is_four_d() {
            self.mode = mode;
        }
    }

    pub fn toggle_four_d(&mut self) {
        self.mode = if self.is_four_d() {
            self.spatial_mode
        } else {
            RenderMode::FourD
        };
    }
}

#[derive(Component)]
pub struct SandboxCamera;

pub fn setup_camera(mut commands: Commands) {
    let spec = render_mode_spec(RenderMode::ThreeD);

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
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut render_mode: ResMut<RenderModeState>,
) {
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        render_mode.set_spatial_mode(RenderMode::ThreeD);
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        render_mode.set_spatial_mode(RenderMode::TwoD);
    } else if keyboard_input.just_pressed(KeyCode::Digit3) {
        render_mode.set_spatial_mode(RenderMode::OneD);
    } else if keyboard_input.just_pressed(KeyCode::Digit4) {
        render_mode.toggle_four_d();
    }
}

pub fn update_view_projection(
    render_mode: Res<RenderModeState>,
    mut camera: Single<&mut Projection, With<SandboxCamera>>,
) {
    **camera = render_mode_spec(render_mode.effective_spatial_mode()).projection;
}

pub fn update_atmosphere(
    time: Res<Time>,
    render_mode: Res<RenderModeState>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    mut fog: Single<&mut DistanceFog, With<SandboxCamera>>,
) {
    let spec = active_render_mode_spec(&render_mode);
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
    render_mode: Res<RenderModeState>,
    mut camera: Single<&mut Transform, (With<SandboxCamera>, Without<SceneRoot>)>,
    mut scene_root: Single<&mut Transform, (With<SceneRoot>, Without<SandboxCamera>)>,
) {
    let spec = render_mode_spec(render_mode.effective_spatial_mode());
    let desired_camera = Transform::from_translation(spec.eye).looking_at(spec.focus, Vec3::Y);

    let camera_blend = smoothing_factor(CAMERA_LERP_SPEED, time.delta_secs());
    camera.translation = camera
        .translation
        .lerp(desired_camera.translation, camera_blend);
    camera.rotation = camera.rotation.slerp(desired_camera.rotation, camera_blend);

    // Keep one shared scene and make each mode legible by compressing axes on the root.
    let root_blend = smoothing_factor(ROOT_LERP_SPEED, time.delta_secs());
    scene_root.scale = scene_root.scale.lerp(spec.scene_scale, root_blend);
}

pub fn active_render_mode_spec(render_mode: &RenderModeState) -> RenderModeSpec {
    let mut spec = render_mode_spec(render_mode.effective_spatial_mode());
    if render_mode.is_four_d() {
        apply_four_d_overlay(&mut spec);
    }
    spec
}

pub fn render_mode_spec(mode: RenderMode) -> RenderModeSpec {
    let mut spec = match mode {
        RenderMode::ThreeD => RenderModeSpec {
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
        RenderMode::TwoD => RenderModeSpec {
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
        RenderMode::OneD => RenderModeSpec {
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
        RenderMode::FourD => RenderModeSpec {
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
    };

    if mode == RenderMode::FourD {
        apply_four_d_overlay(&mut spec);
    }

    spec
}

fn apply_four_d_overlay(spec: &mut RenderModeSpec) {
    spec.clear_color = spec.clear_color.mix(&Color::srgb(0.028, 0.038, 0.062), 0.55);
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::Projection;

    #[test]
    fn render_mode_default_is_3d() {
        assert_eq!(RenderMode::default(), RenderMode::ThreeD);
    }

    #[test]
    fn render_mode_spec_3d_has_perspective_projection() {
        let spec = render_mode_spec(RenderMode::ThreeD);
        assert!(matches!(spec.projection, Projection::Perspective(_)));
        assert_eq!(spec.scene_scale, Vec3::ONE);
    }

    #[test]
    fn render_mode_spec_2d_has_orthographic_projection() {
        let spec = render_mode_spec(RenderMode::TwoD);
        assert!(matches!(spec.projection, Projection::Orthographic(_)));
        assert_eq!(spec.scene_scale.z, TWO_D_DEPTH_SCALE);
    }

    #[test]
    fn render_mode_spec_1d_has_orthographic_projection() {
        let spec = render_mode_spec(RenderMode::OneD);
        assert!(matches!(spec.projection, Projection::Orthographic(_)));
        assert_eq!(spec.scene_scale.y, ONE_D_HEIGHT_SCALE);
        assert_eq!(spec.scene_scale.z, ONE_D_DEPTH_SCALE);
    }

    #[test]
    fn render_mode_spec_1d_is_more_compressed_than_2d() {
        let spec_2d = render_mode_spec(RenderMode::TwoD);
        let spec_1d = render_mode_spec(RenderMode::OneD);
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
    fn render_mode_state_equality() {
        let state1 = RenderModeState {
            mode: RenderMode::ThreeD,
            spatial_mode: RenderMode::ThreeD,
        };
        let state2 = RenderModeState {
            mode: RenderMode::ThreeD,
            spatial_mode: RenderMode::ThreeD,
        };
        let state3 = RenderModeState {
            mode: RenderMode::TwoD,
            spatial_mode: RenderMode::TwoD,
        };
        assert_eq!(state1.mode, state2.mode);
        assert_ne!(state1.mode, state3.mode);
    }

    #[test]
    fn render_mode_clone_is_independent() {
        let state1 = RenderModeState {
            mode: RenderMode::ThreeD,
            spatial_mode: RenderMode::ThreeD,
        };
        let mut state2 = state1.clone();
        state2.mode = RenderMode::OneD;
        assert_eq!(state1.mode, RenderMode::ThreeD);
        assert_eq!(state2.mode, RenderMode::OneD);
    }

    #[test]
    fn four_d_preserves_underlying_spatial_mode() {
        let mut state = RenderModeState::default();
        state.set_spatial_mode(RenderMode::TwoD);
        state.toggle_four_d();

        assert!(state.is_four_d());
        assert_eq!(state.effective_spatial_mode(), RenderMode::TwoD);
    }

    #[test]
    fn active_render_mode_spec_adds_four_d_grade() {
        let mut state = RenderModeState::default();
        state.toggle_four_d();

        let four_d = active_render_mode_spec(&state);
        let three_d = render_mode_spec(RenderMode::ThreeD);
        assert_ne!(four_d.clear_color, three_d.clear_color);
        assert!(four_d.fog_start < three_d.fog_start);
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
