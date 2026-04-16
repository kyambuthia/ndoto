use bevy::{light::CascadeShadowConfigBuilder, prelude::*};

#[derive(Component)]
pub struct SceneRoot;

#[derive(Component)]
pub struct PrimaryPointLight;

#[derive(Component, Clone, Copy, Debug)]
pub struct DreamLight {
    pub(crate) anchor: Vec3,
    pub(crate) radius: f32,
    pub(crate) base_height: f32,
    pub(crate) speed: f32,
    pub(crate) intensity: f32,
    pub(crate) phase: f32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RecordableEntity {
    Floor,
    Cube,
    Wall,
    Pillar,
    Block,
    DreamLight,
}

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.215, 0.24),
        perceptual_roughness: 1.0,
        ..default()
    });
    let structure_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.58, 0.6, 0.63),
        perceptual_roughness: 0.94,
        ..default()
    });
    let accent_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.67, 0.69, 0.74),
        perceptual_roughness: 0.88,
        ..default()
    });

    let root = commands
        .spawn((
            Name::new("SandboxSceneRoot"),
            SceneRoot,
            Transform::default(),
            Visibility::default(),
        ))
        .id();

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Name::new("Floor"),
            RecordableEntity::Floor,
            Mesh3d(meshes.add(Cuboid::new(28.0, 0.2, 28.0))),
            MeshMaterial3d(floor_material.clone()),
            Transform::from_xyz(0.0, -0.1, 0.0),
        ));

        parent.spawn((
            Name::new("Cube"),
            RecordableEntity::Cube,
            Mesh3d(meshes.add(Cuboid::new(1.8, 1.8, 1.8))),
            MeshMaterial3d(structure_material.clone()),
            Transform::from_xyz(-3.4, 0.9, -2.5),
        ));

        parent.spawn((
            Name::new("Wall"),
            RecordableEntity::Wall,
            Mesh3d(meshes.add(Cuboid::new(0.7, 3.0, 7.2))),
            MeshMaterial3d(structure_material),
            Transform::from_xyz(4.0, 1.5, 0.4),
        ));

        parent.spawn((
            Name::new("Pillar"),
            RecordableEntity::Pillar,
            Mesh3d(meshes.add(Cylinder::new(0.7, 3.6).mesh().resolution(48))),
            MeshMaterial3d(accent_material.clone()),
            Transform::from_xyz(0.6, 1.8, 4.6),
        ));

        parent.spawn((
            Name::new("Block"),
            RecordableEntity::Block,
            Mesh3d(meshes.add(Cuboid::new(2.8, 0.8, 2.8))),
            MeshMaterial3d(accent_material),
            Transform::from_xyz(2.1, 0.4, -4.4),
        ));

        parent.spawn((
            Name::new("SunLight"),
            DirectionalLight {
                illuminance: 18_000.0,
                shadows_enabled: true,
                ..default()
            },
            CascadeShadowConfigBuilder {
                first_cascade_far_bound: 10.0,
                maximum_distance: 32.0,
                ..default()
            }
            .build(),
            Transform::from_xyz(6.0, 12.0, 8.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        ));

        parent.spawn((
            Name::new("DreamLight"),
            RecordableEntity::DreamLight,
            PrimaryPointLight,
            DreamLight {
                anchor: Vec3::new(0.0, 0.0, 0.0),
                radius: 6.0,
                base_height: 5.8,
                speed: 0.26,
                intensity: 280_000.0,
                phase: 0.0,
            },
            PointLight {
                intensity: 280_000.0,
                range: 24.0,
                color: Color::srgb(0.74, 0.78, 0.95),
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(5.0, 5.8, 0.0),
        ));
    });
}

pub fn animate_dream_light(
    time: Res<Time>,
    dream_light: Single<
        (&mut DreamLight, &mut Transform, &mut PointLight),
        With<PrimaryPointLight>,
    >,
) {
    let (mut motion, mut transform, mut light) = dream_light.into_inner();
    motion.phase += time.delta_secs() * motion.speed;
    let phase = motion.phase;

    transform.translation = motion.anchor
        + Vec3::new(
            phase.cos() * motion.radius,
            motion.base_height + (phase * 1.7).sin() * 0.35,
            phase.sin() * motion.radius * 0.65,
        );
    light.intensity = motion.intensity + (phase * 1.4).sin() * 30_000.0;
}

const LIGHT_LERP_SPEED: f32 = 4.5;

pub fn update_lighting(
    time: Res<Time>,
    render_mode: Res<crate::rendering::camera::RenderModeState>,
    mut point_light: Single<&mut PointLight, With<PrimaryPointLight>>,
) {
    use crate::rendering::camera::active_render_mode_spec;

    let spec = active_render_mode_spec(&render_mode);
    let blend = 1.0 - (-LIGHT_LERP_SPEED * time.delta_secs()).exp();

    point_light.range += (spec.point_light_range - point_light.range) * blend;
}
