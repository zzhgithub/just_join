use bevy::{
    prelude::{
        shape, App, Assets, BuildChildren, Camera3dBundle, ClearColor, Color, Commands, Component,
        ComputedVisibility, GlobalTransform, Mat4, Mesh, PbrBundle, Plugin, Quat, Res, ResMut,
        StandardMaterial, Startup, Transform, Update, Vec3, Visibility,
    },
    transform::TransformBundle,
};
use bevy_atmosphere::prelude::AtmosphereCamera;
use bevy_rapier3d::prelude::{Collider, ColliderMassProperties, LockedAxes, RigidBody, Sleeping};
use controller::{
    controller::{
        controller_to_pitch, controller_to_yaw, BodyTag, CameraTag, CharacterController, HeadTag,
        YawTag,
    },
    look::{LookDirection, LookEntity},
    rapier::RapierDynamicImpulseCharacterControllerPlugin,
    utils::CharacterSettings,
};

use crate::palyer::{egui_center_cursor_system, PlayerStorge};

// 角色操作使用rapier的插件
pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::hex("101010").unwrap()))
            .add_plugins(RapierDynamicImpulseCharacterControllerPlugin)
            // 这设置了第三人称的视角
            .insert_resource(CharacterSettings {
                focal_point: -Vec3::Z * 10.0,  // Relative to head
                follow_offset: -Vec3::Z * 2.0, // Relative to head
                // 这里是人物的位置
                body_position: Vec3::new(0., 20., 0.),
                ..Default::default()
            })
            // .insert_resource(CharacterSettings::first())
            .add_systems(Startup, spawn_character)
            .add_systems(
                Update,
                (
                    controller_to_yaw,
                    controller_to_pitch,
                    egui_center_cursor_system,
                ),
            );
    }
}

pub fn spawn_character(
    mut commands: Commands,
    character_settings: Res<CharacterSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let box_y = 1.0;
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let red = materials.add(Color::hex("800000").unwrap().into());
    let body = commands
        .spawn((
            GlobalTransform::IDENTITY,
            Transform::IDENTITY,
            CharacterController::default(),
            BodyTag,
            Visibility::Inherited,
            ComputedVisibility::HIDDEN,
            PlayerMe,
        ))
        .insert(RigidBody::Dynamic)
        .insert(Sleeping::default())
        .insert(TransformBundle::from(Transform::from_xyz(
            character_settings.body_position.x,
            character_settings.body_position.y,
            character_settings.body_position.z,
        )))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(ColliderMassProperties::Density(200.0))
        .insert(Collider::capsule(
            (-0.5 * character_settings.scale.y * Vec3::Y).into(),
            (0.5 * character_settings.scale.y * Vec3::Y).into(),
            0.5 * character_settings.scale.x.max(character_settings.scale.z),
        ))
        // .insert(RigidBodyPositionSync::Interpolated { prev_pos: None })
        .id();
    let yaw = commands
        .spawn((
            GlobalTransform::IDENTITY,
            Transform::IDENTITY,
            YawTag,
            Visibility::Inherited,
            ComputedVisibility::HIDDEN,
        ))
        .id();
    let body_model = commands
        .spawn(PbrBundle {
            material: red.clone(),
            mesh: cube.clone(),
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                character_settings.scale - character_settings.head_scale * Vec3::Y,
                Quat::IDENTITY,
                Vec3::new(
                    0.0,
                    0.5 * (box_y + character_settings.scale.y - character_settings.head_scale)
                        - 1.695,
                    0.0,
                ),
            )),
            visibility: Visibility::Visible,
            ..Default::default()
        })
        .id();
    let head = commands
        .spawn((
            GlobalTransform::IDENTITY,
            Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::from_rotation_y(character_settings.head_yaw),
                Vec3::new(
                    0.0,
                    0.5 * (box_y - character_settings.head_scale) + character_settings.scale.y
                        - 1.695,
                    0.0,
                ),
            )),
            HeadTag,
            Visibility::Inherited,
            ComputedVisibility::HIDDEN,
        ))
        .id();
    let head_model = commands
        .spawn(PbrBundle {
            material: red,
            mesh: cube,
            transform: Transform::from_scale(Vec3::splat(character_settings.head_scale)),
            visibility: Visibility::Visible,
            ..Default::default()
        })
        .id();
    let camera = commands
        .spawn(Camera3dBundle {
            transform: Transform::from_matrix(Mat4::look_to_rh(
                character_settings.follow_offset,
                character_settings.focal_point,
                Vec3::Y,
            )),
            ..Default::default()
        })
        .insert(AtmosphereCamera::default())
        .insert((LookDirection::default(), CameraTag))
        .id();
    commands
        .entity(body)
        .insert(LookEntity(camera))
        .push_children(&[yaw]);
    commands.entity(yaw).push_children(&[body_model, head]);
    commands.entity(head).push_children(&[head_model, camera]);

    // 设置头部的检查组件
    // FIXME: 后面可以改成HeadTag进行查询
    commands.insert_resource(PlayerStorge(head));
}

#[derive(Debug, Component)]
pub struct PlayerMe;
