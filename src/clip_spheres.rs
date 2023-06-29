use bevy::prelude::{Component, Query, ResMut, Resource, Transform, Vec3, With};

use crate::VIEW_RADIUS;

#[derive(Debug, Clone, Copy)]
pub struct Sphere3 {
    pub center: Vec3,
    pub radius: f32,
}

#[derive(Debug, Resource, Clone, Copy)]
pub struct ClipSpheres {
    pub old_sphere: Sphere3,
    pub new_sphere: Sphere3,
}

pub fn update_clip_shpere_system<T>(
    mut clip_spheres: ResMut<ClipSpheres>,
    mut query: Query<&mut Transform, With<T>>,
) where
    T: Component,
{
    let position = if let Some(trf) = query.iter().next() {
        trf.translation
    } else {
        return;
    };
    // println!("position update: {:?}", position);1
    clip_spheres.old_sphere = clip_spheres.new_sphere;
    clip_spheres.new_sphere = Sphere3 {
        center: position,
        radius: VIEW_RADIUS,
    }
}
