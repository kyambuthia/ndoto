use bevy::{color::LinearRgba, prelude::*};
use ndoto_framework::{dimension::DimensionState, input::FixedPlayerInput};

use crate::prototype::{
    rendering::scene::{DreamLight, RecordableEntity},
    time::history::{
        DreamLightSnapshot, EntitySnapshot, FrameSnapshot, PlaybackDirection, PointLightSnapshot,
        TimeHistoryState,
    },
};

const TRAIL_GHOST_COUNT: usize = 6;
const TRAIL_SAMPLE_SPACING: usize = 8;

#[derive(Component)]
pub(super) struct TimeIndicatorRoot;

#[derive(Component)]
pub(super) struct TimeIndicatorFill;

#[derive(Component)]
pub(super) struct TimeIndicatorLabel;

#[derive(Component)]
pub(super) struct TimeTrailGhost {
    sample_offset: usize,
}

pub(super) fn setup_time_feedback(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ghost_mesh = meshes.add(Sphere::new(0.18).mesh().uv(20, 12));

    for index in 0..TRAIL_GHOST_COUNT {
        let alpha = 0.22 - index as f32 * 0.025;
        let brightness = 1.9 - index as f32 * 0.18;

        commands.spawn((
            Name::new(format!("TimeTrailGhost{index}")),
            TimeTrailGhost {
                sample_offset: (index + 1) * TRAIL_SAMPLE_SPACING,
            },
            Mesh3d(ghost_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.45, 0.62, 0.92, alpha.max(0.03)),
                emissive: LinearRgba::rgb(0.12 * brightness, 0.18 * brightness, 0.32 * brightness),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_scale(Vec3::splat(1.0)),
            Visibility::Hidden,
        ));
    }
}

pub(super) fn setup_time_indicator(mut commands: Commands) {
    commands
        .spawn((
            Name::new("TimeIndicatorRoot"),
            TimeIndicatorRoot,
            Node {
                position_type: PositionType::Absolute,
                left: percent(20.0),
                bottom: px(22.0),
                width: percent(60.0),
                padding: px(8.0).all(),
                flex_direction: FlexDirection::Column,
                row_gap: px(6.0),
                border_radius: BorderRadius::all(px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.05, 0.08, 0.72)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100.0),
                        height: px(6.0),
                        border: px(1.0).all(),
                        border_radius: BorderRadius::all(px(999.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.09, 0.13, 0.95)),
                    BorderColor::all(Color::srgba(0.42, 0.52, 0.7, 0.55)),
                ))
                .with_children(|track| {
                    track.spawn((
                        TimeIndicatorFill,
                        Node {
                            width: percent(100.0),
                            height: percent(100.0),
                            border_radius: BorderRadius::all(px(999.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.52, 0.68, 0.94, 0.85)),
                    ));
                });

            parent.spawn((
                TimeIndicatorLabel,
                Text::new("4D time dormant"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.76, 0.82, 0.9)),
            ));
        });
}

pub(super) fn update_time_mode(
    player_input: Res<FixedPlayerInput>,
    mut dimension_state: ResMut<DimensionState>,
    mut history_state: ResMut<TimeHistoryState>,
) {
    if player_input.toggle_four_d {
        dimension_state.toggle_four_d();
    }

    history_state.playback_direction = if dimension_state.is_four_d() {
        if player_input.rewind {
            Some(PlaybackDirection::Rewind)
        } else if player_input.fast_forward {
            Some(PlaybackDirection::Forward)
        } else {
            None
        }
    } else {
        None
    };
    history_state.recording = history_state.playback_direction.is_none();
}

pub(super) fn record_state(
    mut history_state: ResMut<TimeHistoryState>,
    recordables: Query<(
        &RecordableEntity,
        &Transform,
        Option<&PointLight>,
        Option<&DreamLight>,
    )>,
) {
    if !history_state.recording {
        return;
    }

    if recordables.is_empty() {
        return;
    }

    if history_state.cursor < history_state.newest_cursor() {
        let rewind_index = history_state.cursor.floor() as usize;
        history_state.history.truncate_after(rewind_index);
    }

    let mut entities = recordables
        .iter()
        .map(|(id, transform, point_light, dream_light)| EntitySnapshot {
            id: *id,
            transform: *transform,
            point_light: point_light.map(|light| PointLightSnapshot {
                intensity: light.intensity,
                range: light.range,
            }),
            dream_light: dream_light.map(|dream_light| DreamLightSnapshot {
                anchor: dream_light.anchor,
                radius: dream_light.radius,
                base_height: dream_light.base_height,
                speed: dream_light.speed,
                intensity: dream_light.intensity,
                phase: dream_light.phase,
            }),
        })
        .collect::<Vec<_>>();

    entities.sort_by_key(|entity| entity.id);

    history_state.history.push(FrameSnapshot { entities });
    history_state.cursor = history_state.newest_cursor();
}

pub(super) fn playback_state(
    time: Res<Time<Fixed>>,
    mut history_state: ResMut<TimeHistoryState>,
    mut recordables: Query<(
        &RecordableEntity,
        &mut Transform,
        Option<&mut PointLight>,
        Option<&mut DreamLight>,
    )>,
) {
    let Some(direction) = history_state.playback_direction else {
        return;
    };
    if history_state.history.is_empty() {
        return;
    }

    let direction_scalar = match direction {
        PlaybackDirection::Rewind => -1.0,
        PlaybackDirection::Forward => 1.0,
    };

    history_state.cursor += direction_scalar * direction.frames_per_second() * time.delta_secs();
    history_state.cursor = history_state
        .cursor
        .clamp(0.0, history_state.newest_cursor());

    let Some(sample) = history_state.history.sample(history_state.cursor) else {
        return;
    };

    for (id, mut transform, point_light, dream_light) in &mut recordables {
        let Some(from) = sample.from.get(*id) else {
            continue;
        };
        let Some(to) = sample.to.get(*id) else {
            continue;
        };

        apply_transform_sample(&mut transform, from, to, sample.blend);

        if let (Some(mut light), Some(from_light), Some(to_light)) = (
            point_light,
            from.point_light.as_ref(),
            to.point_light.as_ref(),
        ) {
            light.intensity = lerp(from_light.intensity, to_light.intensity, sample.blend);
            light.range = lerp(from_light.range, to_light.range, sample.blend);
        }

        if let (Some(mut motion), Some(from_motion), Some(to_motion)) = (
            dream_light,
            from.dream_light.as_ref(),
            to.dream_light.as_ref(),
        ) {
            apply_dream_light_sample(&mut motion, from_motion, to_motion, sample.blend);
        }
    }
}

pub(super) fn update_time_indicator(
    dimension_state: Res<DimensionState>,
    history_state: Res<TimeHistoryState>,
    mut root: Single<&mut Visibility, With<TimeIndicatorRoot>>,
    mut fill: Single<&mut Node, With<TimeIndicatorFill>>,
    mut label: Single<&mut Text, With<TimeIndicatorLabel>>,
) {
    **root = if dimension_state.is_four_d() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    fill.width = percent(history_state.timeline_fraction() * 100.0);

    let state_label = match history_state.playback_direction {
        Some(PlaybackDirection::Rewind) => "rewind",
        Some(PlaybackDirection::Forward) => "fast-forward",
        None => "recording",
    };

    let seconds = history_state.history.len() as f32 / 60.0;
    let cursor_seconds = history_state.cursor / 60.0;
    **label = Text::new(format!(
        "4D {state_label}  {cursor_seconds:04.1}s / {seconds:04.1}s   hold Space/R to rewind, F to fast-forward"
    ));
}

pub(super) fn update_time_trails(
    dimension_state: Res<DimensionState>,
    history_state: Res<TimeHistoryState>,
    mut ghosts: Query<(&TimeTrailGhost, &mut Transform, &mut Visibility)>,
) {
    for (ghost, mut transform, mut visibility) in &mut ghosts {
        if !dimension_state.is_four_d() || history_state.history.len() <= ghost.sample_offset {
            *visibility = Visibility::Hidden;
            continue;
        }

        let cursor = (history_state.cursor - ghost.sample_offset as f32).max(0.0);
        let Some(sample) = history_state.history.sample(cursor) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let Some(from) = sample.from.get(RecordableEntity::DreamLight) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let Some(to) = sample.to.get(RecordableEntity::DreamLight) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        transform.translation = from
            .transform
            .translation
            .lerp(to.transform.translation, sample.blend);
        let scale = (1.0 - ghost.sample_offset as f32 * 0.01).max(0.35);
        transform.scale = Vec3::splat(scale);
        *visibility = Visibility::Inherited;
    }
}

fn apply_transform_sample(
    transform: &mut Transform,
    from: &EntitySnapshot,
    to: &EntitySnapshot,
    blend: f32,
) {
    transform.translation = from
        .transform
        .translation
        .lerp(to.transform.translation, blend);
    transform.rotation = from.transform.rotation.slerp(to.transform.rotation, blend);
    transform.scale = from.transform.scale.lerp(to.transform.scale, blend);
}

fn apply_dream_light_sample(
    motion: &mut DreamLight,
    from: &DreamLightSnapshot,
    to: &DreamLightSnapshot,
    blend: f32,
) {
    motion.anchor = from.anchor.lerp(to.anchor, blend);
    motion.radius = lerp(from.radius, to.radius, blend);
    motion.base_height = lerp(from.base_height, to.base_height, blend);
    motion.speed = lerp(from.speed, to.speed, blend);
    motion.intensity = lerp(from.intensity, to.intensity, blend);
    motion.phase = lerp(from.phase, to.phase, blend);
}

fn lerp(start: f32, end: f32, blend: f32) -> f32 {
    start + (end - start) * blend
}
