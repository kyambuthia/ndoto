use bevy::prelude::*;
use ndoto_engine::{
    BackendQueryHit, PhysicsBackendState, PhysicsRayCastRequest, PhysicsShapeCastRequest,
};

use crate::dimension::{DimensionState, SpatialMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AxisMask {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl AxisMask {
    pub const XYZ: Self = Self {
        x: true,
        y: true,
        z: true,
    };
    pub const XY: Self = Self {
        x: true,
        y: true,
        z: false,
    };
    pub const X: Self = Self {
        x: true,
        y: false,
        z: false,
    };

    pub fn project(self, value: Vec3) -> Vec3 {
        Vec3::new(
            if self.x { value.x } else { 0.0 },
            if self.y { value.y } else { 0.0 },
            if self.z { value.z } else { 0.0 },
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DimensionQueryContext {
    pub spatial_mode: SpatialMode,
}

impl From<&DimensionState> for DimensionQueryContext {
    fn from(value: &DimensionState) -> Self {
        Self {
            spatial_mode: value.spatial_mode,
        }
    }
}

impl DimensionQueryContext {
    pub fn query_axes(self) -> AxisMask {
        match self.spatial_mode {
            SpatialMode::ThreeD => AxisMask::XYZ,
            SpatialMode::TwoD => AxisMask::XY,
            SpatialMode::OneD => AxisMask::X,
        }
    }

    pub fn project_direction(self, direction: Vec3) -> Vec3 {
        self.query_axes().project(direction)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DimensionRayCastRequest {
    pub context: DimensionQueryContext,
    pub origin: Vec3,
    pub direction: Vec3,
    pub max_distance: f32,
    pub collision_mask: u32,
}

impl DimensionRayCastRequest {
    pub fn to_backend(self) -> PhysicsRayCastRequest {
        PhysicsRayCastRequest {
            origin: self.context.query_axes().project(self.origin),
            direction: self.context.project_direction(self.direction),
            max_distance: self.max_distance,
            collision_mask: self.collision_mask,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DimensionShapeCastRequest {
    pub context: DimensionQueryContext,
    pub origin: Vec3,
    pub translation: Vec3,
    pub radius: f32,
    pub half_height: f32,
    pub collision_mask: u32,
}

impl DimensionShapeCastRequest {
    pub fn to_backend(self) -> PhysicsShapeCastRequest {
        PhysicsShapeCastRequest {
            origin: self.context.query_axes().project(self.origin),
            translation: self.context.project_direction(self.translation),
            radius: self.radius,
            half_height: self.half_height,
            collision_mask: self.collision_mask,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicsApiState {
    pub backend: PhysicsBackendState,
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsApiState>()
            .add_systems(Startup, sync_backend_state);
    }
}

impl Default for PhysicsApiState {
    fn default() -> Self {
        Self {
            backend: PhysicsBackendState::default(),
        }
    }
}

fn sync_backend_state(
    backend_state: Res<PhysicsBackendState>,
    mut physics_api_state: ResMut<PhysicsApiState>,
) {
    physics_api_state.backend = *backend_state;
}

pub fn project_backend_hit(
    context: DimensionQueryContext,
    hit: BackendQueryHit,
) -> BackendQueryHit {
    let axes = context.query_axes();
    BackendQueryHit {
        entity: hit.entity,
        position: axes.project(hit.position),
        normal: axes.project(hit.normal),
        distance: hit.distance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_axes_match_spatial_mode() {
        assert_eq!(
            DimensionQueryContext {
                spatial_mode: SpatialMode::ThreeD
            }
            .query_axes(),
            AxisMask::XYZ
        );
        assert_eq!(
            DimensionQueryContext {
                spatial_mode: SpatialMode::TwoD
            }
            .query_axes(),
            AxisMask::XY
        );
        assert_eq!(
            DimensionQueryContext {
                spatial_mode: SpatialMode::OneD
            }
            .query_axes(),
            AxisMask::X
        );
    }

    #[test]
    fn raycast_projection_zeroes_inactive_axes() {
        let request = DimensionRayCastRequest {
            context: DimensionQueryContext {
                spatial_mode: SpatialMode::OneD,
            },
            origin: Vec3::new(1.0, 2.0, 3.0),
            direction: Vec3::new(4.0, 5.0, 6.0),
            max_distance: 10.0,
            collision_mask: 7,
        };

        let backend = request.to_backend();

        assert_eq!(backend.origin, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(backend.direction, Vec3::new(4.0, 0.0, 0.0));
    }
}
