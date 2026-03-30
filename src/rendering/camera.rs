use bevy::{camera::ScalingMode, prelude::*};

use crate::rendering::scene::SceneRoot;

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
}

#[derive(Resource, Debug, Default)]
pub struct RenderModeState {
    pub mode: RenderMode,
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
        render_mode.mode = RenderMode::ThreeD;
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        render_mode.mode = RenderMode::TwoD;
    } else if keyboard_input.just_pressed(KeyCode::Digit3) {
        render_mode.mode = RenderMode::OneD;
    }
}

pub fn update_view_projection(
    render_mode: Res<RenderModeState>,
    mut camera: Single<&mut Projection, With<SandboxCamera>>,
) {
    **camera = render_mode_spec(render_mode.mode).projection;
}

pub fn animate_view(
    time: Res<Time>,
    render_mode: Res<RenderModeState>,
    mut camera: Single<&mut Transform, (With<SandboxCamera>, Without<SceneRoot>)>,
    mut scene_root: Single<&mut Transform, (With<SceneRoot>, Without<SandboxCamera>)>,
) {
    let spec = render_mode_spec(render_mode.mode);
    let desired_camera = Transform::from_translation(spec.eye).looking_at(spec.focus, Vec3::Y);

    let camera_blend = smoothing_factor(CAMERA_LERP_SPEED, time.delta_secs());
    camera.translation = camera
        .translation
        .lerp(desired_camera.translation, camera_blend);
    camera.rotation = camera.rotation.slerp(desired_camera.rotation, camera_blend);

    let root_blend = smoothing_factor(ROOT_LERP_SPEED, time.delta_secs());
    scene_root.scale = scene_root.scale.lerp(spec.scene_scale, root_blend);
}

fn render_mode_spec(mode: RenderMode) -> RenderModeSpec {
    match mode {
        RenderMode::ThreeD => RenderModeSpec {
            eye: Vec3::new(11.5, 8.5, 13.0),
            focus: Vec3::new(0.0, 1.6, 0.0),
            scene_scale: Vec3::ONE,
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 50.0_f32.to_radians(),
                ..default()
            }),
        },
        RenderMode::TwoD => RenderModeSpec {
            eye: Vec3::new(0.0, 8.0, 18.0),
            focus: Vec3::new(0.0, 1.5, 0.0),
            scene_scale: Vec3::new(1.0, 1.0, TWO_D_DEPTH_SCALE),
            projection: Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: 10.5,
                },
                ..OrthographicProjection::default_3d()
            }),
        },
        RenderMode::OneD => RenderModeSpec {
            eye: Vec3::new(0.0, 2.3, 20.0),
            focus: Vec3::new(0.0, 2.3, 0.0),
            scene_scale: Vec3::new(1.0, ONE_D_HEIGHT_SCALE, ONE_D_DEPTH_SCALE),
            projection: Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: 4.2,
                },
                ..OrthographicProjection::default_3d()
            }),
        },
    }
}

fn smoothing_factor(speed: f32, delta_seconds: f32) -> f32 {
    1.0 - (-speed * delta_seconds).exp()
}

struct RenderModeSpec {
    eye: Vec3,
    focus: Vec3,
    scene_scale: Vec3,
    projection: Projection,
}
