use bevy::prelude::{Component, Resource, Vec3};

#[derive(Resource)]
pub struct CharacterSettings {
    pub scale: Vec3,
    pub head_scale: f32,
    pub head_yaw: f32,
    pub follow_offset: Vec3,
    pub focal_point: Vec3,
    pub body_position: Vec3,
}

impl Default for CharacterSettings {
    fn default() -> Self {
        Self {
            scale: Vec3::new(0.5, 1.9, 0.3),
            head_scale: 0.3,
            head_yaw: 0.0,
            follow_offset: Vec3::new(0.0, 4.0, 8.0), // Relative to head
            focal_point: Vec3::ZERO,                 // Relative to head
            body_position: Vec3::ZERO,
        }
    }
}

impl CharacterSettings {
    pub fn first() -> Self {
        Self {
            focal_point: -Vec3::Z,     // Relative to head
            follow_offset: Vec3::ZERO, // Relative to head
            ..Default::default()
        }
    }
}

#[derive(Component)]
pub struct FakeKinematicRigidBody;
