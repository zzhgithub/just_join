use bevy::{
    prelude::{
        shape, App, Assets, BuildChildren, Camera3dBundle, ClearColor, Color, Commands,
        ComputedVisibility, GlobalTransform, Mat4, Mesh, Msaa, PbrBundle, PointLightBundle, Quat,
        Res, ResMut, SpatialBundle, StandardMaterial, Startup, Transform, Update, Vec3, Visibility,
    },
    transform::TransformBundle,
    utils::petgraph::visit::Visitable,
    DefaultPlugins,
};
use bevy_rapier3d::{
    prelude::{
        Collider, ColliderMassProperties, LockedAxes, NoUserData, RapierConfiguration,
        RapierPhysicsPlugin, RigidBody, Sleeping, TimestepMode,
    },
    render::RapierDebugRenderPlugin,
};
use controller::{
    controller::{
        controller_to_pitch, controller_to_yaw, BodyTag, CameraTag, CharacterController, HeadTag,
        YawTag,
    },
    look::{LookDirection, LookEntity},
    rapier::{
        RapierDynamicForceCharacterControllerPlugin, RapierDynamicImpulseCharacterControllerPlugin,
    },
    utils::CharacterSettings,
};
use rand::Rng;

pub fn main() {
    let mut app_builder = App::new();
    app_builder
        .insert_resource(ClearColor(Color::hex("101010").unwrap()))
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        // 这里使用了物理系统
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // debug
        .add_plugins(RapierDebugRenderPlugin::default())
        // .add_plugins(RapierDynamicForceCharacterControllerPlugin)
        .add_plugins(RapierDynamicImpulseCharacterControllerPlugin)
        .insert_resource(CharacterSettings {
            focal_point: -Vec3::Z * 10.0,  // Relative to head
            follow_offset: -Vec3::Z * 2.0, // Relative to head
            ..Default::default()
        })
        // .insert_resource(CharacterSettings::first())
        // 设置时间类型
        .insert_resource(RapierConfiguration {
            timestep_mode: TimestepMode::Interpolated {
                dt: 1. / 60.,
                time_scale: 1.,
                substeps: 1,
            },
            ..Default::default()
        })
        .add_systems(Startup, (spawn_world, spawn_character))
        .add_systems(Update, (controller_to_yaw, controller_to_pitch))
        .run();
}

pub fn spawn_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    // Light
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(-15.0, 10.0, -15.0)),
        ..Default::default()
    });

    // Ground cuboid
    let grey = materials.add(Color::hex("808080").unwrap().into());
    let box_xz = 200.0;
    let box_y = 1.0;
    commands
        .spawn(PbrBundle {
            material: grey,
            mesh: cube.clone(),
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::new(box_xz, box_y, box_xz),
                Quat::IDENTITY,
                Vec3::ZERO,
            )),
            ..Default::default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(0.5 * box_xz, 0.5 * box_y, 0.5 * box_xz));

    // Cubes for some kind of reference in the scene to make it easy to see
    // what is happening
    let teal = materials.add(Color::hex("008080").unwrap().into());
    let cube_scale = 1.0;
    let mut rng = rand::thread_rng();
    for _ in 0..20 {
        let x = rng.gen_range(-10.0..10.0);
        let z = rng.gen_range(-10.0..10.0);
        let translation = Vec3::new(x, 0.5 * (cube_scale - box_y), z);

        let body = translation + Vec3::new(x, 0.5 * (cube_scale - box_y), z);
        commands
            .spawn(PbrBundle {
                material: teal.clone(),
                mesh: cube.clone(),
                transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                    Vec3::splat(cube_scale),
                    Quat::IDENTITY,
                    translation,
                )),
                ..Default::default()
            })
            .insert(RigidBody::Dynamic)
            .insert(Sleeping::default())
            .insert(TransformBundle::from(Transform::from_xyz(
                body.x, body.y, body.z,
            )))
            .insert(Collider::cuboid(
                0.5 * cube_scale,
                0.5 * cube_scale,
                0.5 * cube_scale,
            ));
        // .insert(RigidBodyPositionSync::Interpolated { prev_pos: None });
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

    let body_tmf = 0.5 * (box_y + character_settings.scale.y) * Vec3::Y;

    let body = commands
        .spawn((
            GlobalTransform::IDENTITY,
            Transform::IDENTITY,
            CharacterController::default(),
            BodyTag,
            Visibility::Inherited,
            ComputedVisibility::HIDDEN,
        ))
        .insert(RigidBody::Dynamic)
        .insert(Sleeping::default())
        .insert(TransformBundle::from(Transform::from_xyz(
            body_tmf.x, body_tmf.y, body_tmf.z,
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
            ComputedVisibility::HIDDEN
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
        .insert((LookDirection::default(), CameraTag))
        .id();
    commands
        .entity(body)
        .insert(LookEntity(camera))
        .push_children(&[yaw]);
    commands.entity(yaw).push_children(&[body_model, head]);
    commands.entity(head).push_children(&[head_model, camera]);
}
