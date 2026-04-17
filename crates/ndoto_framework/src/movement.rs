use bevy::prelude::*;
use ndoto_engine::EngineFixedSet;

use crate::{
    dimension::DimensionState,
    input::FixedPlayerInput,
    physics::{BoxCollider, DimensionQueryContext},
};

#[derive(Component)]
pub struct PlayerControlled;

#[derive(Component, Clone, Copy, Debug)]
pub struct LocomotionConfig {
    pub walk_speed: f32,
    pub sprint_multiplier: f32,
    pub jump_velocity: f32,
    pub gravity: f32,
    pub ground_height: f32,
    pub body_radius: f32,
    pub body_half_height: f32,
}

impl Default for LocomotionConfig {
    fn default() -> Self {
        Self {
            walk_speed: 4.5,
            sprint_multiplier: 1.65,
            jump_velocity: 5.6,
            gravity: 18.0,
            ground_height: 0.0,
            body_radius: 0.45,
            body_half_height: 0.9,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct MovementVelocity {
    pub linear: Vec3,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Grounded(pub bool);

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            apply_player_locomotion.in_set(EngineFixedSet::Simulation),
        );
    }
}

pub fn apply_player_locomotion(
    time: Res<Time<Fixed>>,
    dimension_state: Res<DimensionState>,
    player_input: Res<FixedPlayerInput>,
    colliders: Query<(&Transform, &BoxCollider), Without<PlayerControlled>>,
    mut players: Query<
        (
            &LocomotionConfig,
            &mut Transform,
            &mut MovementVelocity,
            &mut Grounded,
        ),
        With<PlayerControlled>,
    >,
) {
    let delta_seconds = time.delta_secs();

    for (config, mut transform, mut velocity, mut grounded) in &mut players {
        let horizontal_axis = constrained_move_axis(
            player_input.move_axis,
            DimensionQueryContext::from(dimension_state.as_ref()),
        );
        let speed = if player_input.sprint {
            config.walk_speed * config.sprint_multiplier
        } else {
            config.walk_speed
        };

        velocity.linear.x = horizontal_axis.x * speed;
        velocity.linear.z = horizontal_axis.z * speed;

        if grounded.0 && player_input.jump {
            velocity.linear.y = config.jump_velocity;
            grounded.0 = false;
        } else if !grounded.0 {
            velocity.linear.y -= config.gravity * delta_seconds;
        }

        transform.translation += velocity.linear * delta_seconds;

        let query_context = DimensionQueryContext::from(dimension_state.as_ref());
        let projected_translation = query_context.project_direction(Vec3::new(
            transform.translation.x,
            0.0,
            transform.translation.z,
        ));
        transform.translation.x = projected_translation.x;
        transform.translation.z = projected_translation.z;

        let projected_velocity =
            query_context.project_direction(Vec3::new(velocity.linear.x, 0.0, velocity.linear.z));
        velocity.linear.x = projected_velocity.x;
        velocity.linear.z = projected_velocity.z;

        resolve_static_collisions(
            &mut transform.translation,
            config.body_radius,
            config.body_half_height,
            colliders.iter(),
        );

        let grounded_height = config.ground_height + config.body_half_height;
        if transform.translation.y <= grounded_height {
            transform.translation.y = grounded_height;
            velocity.linear.y = 0.0;
            grounded.0 = true;
        } else {
            grounded.0 = false;
        }
    }
}

fn constrained_move_axis(move_axis: IVec2, query_context: DimensionQueryContext) -> Vec3 {
    let wish = Vec2::new(move_axis.x as f32, move_axis.y as f32).clamp_length_max(1.0);
    query_context.project_direction(Vec3::new(wish.x, 0.0, wish.y))
}

fn resolve_static_collisions<'a>(
    player_position: &mut Vec3,
    body_radius: f32,
    body_half_height: f32,
    colliders: impl Iterator<Item = (&'a Transform, &'a BoxCollider)>,
) {
    for (transform, collider) in colliders {
        let min = transform.translation - collider.half_extents;
        let max = transform.translation + collider.half_extents;

        let player_bottom = player_position.y - body_half_height;
        let player_top = player_position.y + body_half_height;
        if player_top <= min.y || player_bottom >= max.y {
            continue;
        }

        let closest = Vec2::new(
            player_position.x.clamp(min.x, max.x),
            player_position.z.clamp(min.z, max.z),
        );
        let player_xz = Vec2::new(player_position.x, player_position.z);
        let delta = player_xz - closest;
        let distance_squared = delta.length_squared();
        if distance_squared >= body_radius * body_radius {
            continue;
        }

        if distance_squared > f32::EPSILON {
            let distance = distance_squared.sqrt();
            let push = delta / distance * (body_radius - distance);
            player_position.x += push.x;
            player_position.z += push.y;
            continue;
        }

        let expand_x = (max.x - min.x) * 0.5 + body_radius;
        let expand_z = (max.z - min.z) * 0.5 + body_radius;
        let offset_x = player_position.x - transform.translation.x;
        let offset_z = player_position.z - transform.translation.z;
        let penetration_x = expand_x - offset_x.abs();
        let penetration_z = expand_z - offset_z.abs();

        if penetration_x < penetration_z {
            player_position.x += penetration_x.copysign(offset_x.signum());
        } else {
            player_position.z += penetration_z.copysign(offset_z.signum());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constrained_move_axis_matches_spatial_mode() {
        assert_eq!(
            constrained_move_axis(
                IVec2::new(1, 1),
                DimensionQueryContext {
                    spatial_mode: crate::dimension::SpatialMode::ThreeD
                }
            ),
            Vec3::new(
                std::f32::consts::FRAC_1_SQRT_2,
                0.0,
                std::f32::consts::FRAC_1_SQRT_2
            )
        );
        assert_eq!(
            constrained_move_axis(
                IVec2::new(1, 1),
                DimensionQueryContext {
                    spatial_mode: crate::dimension::SpatialMode::TwoD
                }
            ),
            Vec3::new(std::f32::consts::FRAC_1_SQRT_2, 0.0, 0.0)
        );
        assert_eq!(
            constrained_move_axis(
                IVec2::new(-1, 1),
                DimensionQueryContext {
                    spatial_mode: crate::dimension::SpatialMode::OneD
                }
            ),
            Vec3::new(-std::f32::consts::FRAC_1_SQRT_2, 0.0, 0.0)
        );
    }

    #[test]
    fn resolve_static_collisions_pushes_player_out_of_box() {
        let collider_transform = Transform::from_xyz(0.0, 1.0, 0.0);
        let collider = BoxCollider::new(Vec3::new(1.0, 1.0, 1.0));
        let mut player_position = Vec3::new(0.2, 0.9, 0.0);

        resolve_static_collisions(
            &mut player_position,
            0.45,
            0.9,
            [(&collider_transform, &collider)].into_iter(),
        );

        assert!(player_position.x > 1.0);
    }
}
