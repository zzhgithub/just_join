use bevy::{
    prelude::{
        shape, App, Assets, BuildChildren, Camera3dBundle, ClearColor, Color, Commands, Component,
        EventReader, GlobalTransform, Input, KeyCode, Mat4, Mesh, Msaa, PbrBundle, PointLight,
        PointLightBundle, PreUpdate, Quat, Query, Res, ResMut, Resource, SpotLight,
        SpotLightBundle, StandardMaterial, Startup, Transform, Update, Vec3, With, Without,
    },
    DefaultPlugins,
};
use controller::{
    controller::{
        controller_to_pitch, controller_to_yaw, BodyTag, CameraTag, CharacterController,
        CharacterControllerPlugin, HeadTag, Mass, YawTag,
    },
    events::TranslationEvent,
    look::{LookDirection, LookEntity},
};
use rand::Rng;

#[derive(Resource)]
pub struct CharacterSettings {
    pub scale: Vec3,
    pub head_scale: f32,
    pub head_yaw: f32,
    pub follow_offset: Vec3,
    pub focal_point: Vec3,
}

impl Default for CharacterSettings {
    fn default() -> Self {
        Self {
            scale: Vec3::new(0.5, 1.9, 0.3),
            head_scale: 0.3,
            head_yaw: 0.0,
            follow_offset: Vec3::new(0.0, 4.0, 8.0), // Relative to head
            focal_point: Vec3::ZERO,                 // Relative to head
        }
    }
}

impl CharacterSettings {
    fn first() -> Self {
        Self {
            focal_point: -Vec3::Z,     // Relative to head
            follow_offset: Vec3::ZERO, // Relative to head
            ..Default::default()
        }
    }
}

#[derive(Component)]
pub struct FakeKinematicRigidBody;

pub fn spawn_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    // Light
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(2.0, 20.0, 2.0)),
        ..Default::default()
    });
    // Ground cuboidd
    let grey = materials.add(Color::hex("808080").unwrap().into());
    commands.spawn(PbrBundle {
        material: grey,
        mesh: cube.clone(),
        transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
            Vec3::new(20.0, 1.0, 20.0),
            Quat::IDENTITY,
            -Vec3::Y,
        )),
        ..Default::default()
    });

    // Cubes for some kind of reference in the scene to make it easy to see
    // what is happening
    let teal = materials.add(Color::hex("008080").unwrap().into());
    let cube_scale = 0.25;
    let mut rng = rand::thread_rng();
    for _ in 0..20 {
        let x = rng.gen_range(-10.0..10.0);
        let z = rng.gen_range(-10.0..10.0);
        commands.spawn(PbrBundle {
            material: teal.clone(),
            mesh: cube.clone(),
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::splat(cube_scale),
                Quat::IDENTITY,
                Vec3::new(x, 0.5 * (cube_scale - 1.0), z),
            )),
            ..Default::default()
        });
    }
}

pub fn spawn_character(
    mut commands: Commands,
    character_settings: Res<CharacterSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let red = materials.add(Color::hex("800000").unwrap().into());

    let body = commands
        .spawn((
            GlobalTransform::IDENTITY,
            Transform::IDENTITY,
            CharacterController::default(),
            FakeKinematicRigidBody,
            Mass::new(80.0),
            BodyTag,
        ))
        .id();
    let yaw = commands
        .spawn((GlobalTransform::IDENTITY, Transform::IDENTITY, YawTag))
        .id();
    let body_model = commands
        .spawn(PbrBundle {
            material: red.clone(),
            mesh: cube.clone(),
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                character_settings.scale - character_settings.head_scale * Vec3::Y,
                Quat::IDENTITY,
                Vec3::new(0.0, character_settings.head_scale, 0.0),
            )),
            ..Default::default()
        })
        .id();
    let head = commands
        .spawn((
            GlobalTransform::IDENTITY,
            Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::from_rotation_y(character_settings.head_yaw),
                (0.5 * character_settings.scale.y + character_settings.head_scale) * Vec3::Y,
            )),
            HeadTag,
        ))
        .id();
    let head_model = commands
        .spawn(PbrBundle {
            material: red,
            mesh: cube,
            transform: Transform::from_scale(Vec3::splat(character_settings.head_scale)),
            ..Default::default()
        })
        .id();
    let camera = commands
        .spawn(Camera3dBundle {
            transform: Transform::from_matrix(Mat4::look_at_rh(
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

pub fn controller_to_kinematic(
    mut translations: EventReader<TranslationEvent>,
    mut query: Query<
        (&mut Transform, &mut CharacterController),
        (With<BodyTag>, With<FakeKinematicRigidBody>),
    >,
) {
    for (mut transform, mut controller) in query.iter_mut() {
        for translation in translations.iter() {
            transform.translation += **translation;
        }
        // NOTE: This is just an example to stop falling past the initial body height
        // With a physics engine you would indicate that the body has collided with
        // something and should stop, depending on how your game works.
        if transform.translation.y < 0.0 {
            transform.translation.y = 0.0;
            controller.jumping = false;
        }
    }
}

pub fn main() {
    let mut app_builder = App::new();
    app_builder
        .insert_resource(ClearColor(Color::hex("101010").unwrap()))
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugins(CharacterControllerPlugin)
        // .add_system(exit_on_esc_system)
        .add_systems(Startup, (spawn_world, spawn_character))
        // .insert_resource(CharacterSettings {
        //     focal_point: Vec3::new(0., 0., 20.0),     // Relative to head
        //     follow_offset: Vec3::ZERO, // Relative to head
        //     ..Default::default()
        // })
        .insert_resource(CharacterSettings::first())
        .add_systems(PreUpdate, toggle_look_mode)
        .add_systems(
            Update,
            (
                controller_to_kinematic,
                controller_to_yaw,
                controller_to_pitch,
                light_follow_camera_system::<BodyTag>,
            ),
        )
        .run();
}

fn toggle_look_mode(mut settings: ResMut<CharacterSettings>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::L) {
        *settings = CharacterSettings::default();
    }
    if input.just_pressed(KeyCode::P) {
        *settings = CharacterSettings::first();
    }
}

fn light_follow_camera_system<T>(
    cam_q: Query<&Transform, With<T>>,
    mut light_q: Query<&mut Transform, (With<PointLight>, Without<T>)>,
) where
    T: Component,
{
    let camera_position = if let Some(tfm) = cam_q.iter().next() {
        tfm.translation
    } else {
        return;
    };
    if let Some(mut tfm) = light_q.iter_mut().next() {
        tfm.translation = camera_position;
    }
}
