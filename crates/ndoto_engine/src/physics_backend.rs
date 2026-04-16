use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicsBackendKind {
    Jolt,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicsBackendState {
    pub kind: PhysicsBackendKind,
}

impl Default for PhysicsBackendState {
    fn default() -> Self {
        Self {
            kind: PhysicsBackendKind::Jolt,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PhysicsBodyHandle(pub u64);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsRayCastRequest {
    pub origin: Vec3,
    pub direction: Vec3,
    pub max_distance: f32,
    pub collision_mask: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsShapeCastRequest {
    pub origin: Vec3,
    pub translation: Vec3,
    pub radius: f32,
    pub half_height: f32,
    pub collision_mask: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackendQueryHit {
    pub entity: Entity,
    pub position: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

pub struct PhysicsBackendPlugin;

impl Plugin for PhysicsBackendPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsBackendState>();
    }
}
