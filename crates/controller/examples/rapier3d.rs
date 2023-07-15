use bevy::{
    prelude::{App, ClearColor, Color, Msaa, Update, Vec3},
    DefaultPlugins,
};
use bevy_rapier3d::prelude::{NoUserData, RapierConfiguration, RapierPhysicsPlugin, TimestepMode};
use controller::{
    controller::{controller_to_pitch, controller_to_yaw},
    rapier::RapierDynamicImpulseCharacterControllerPlugin,
    utils::CharacterSettings,
};

pub fn main() {
    let mut app_builder = App::new();
    app_builder
        .insert_resource(ClearColor(Color::hex("101010").unwrap()))
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDynamicImpulseCharacterControllerPlugin)
        .insert_resource(CharacterSettings {
            focal_point: -Vec3::Z * 10.0,  // Relative to head
            follow_offset: -Vec3::Z * 2.0, // Relative to head
            ..Default::default()
        })
        .add_systems(Update, (controller_to_yaw, controller_to_pitch));
}
