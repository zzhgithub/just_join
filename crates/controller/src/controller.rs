use bevy::{
    prelude::{
        warn, Component, EventReader, EventWriter, Input, IntoSystemConfigs, IntoSystemSetConfigs,
        KeyCode, Plugin, PreUpdate, Quat, Query, Res, Startup, SystemSet, Transform, Vec3, With,
    },
    time::Time,
    window::{CursorGrabMode, PrimaryWindow, Window},
};

use crate::{
    events::{
        ForceEvent, ImpulseEvent, LookDeltaEvent, LookEvent, PitchEvent, TranslationEvent, YawEvent,
    },
    input_map::InputMap,
    look::{forward_up, input_to_look, LookDirection, LookEntity, MouseSettings},
};

#[derive(Debug, Component)]
pub struct BodyTag;
// 首摇
#[derive(Debug, Component)]
pub struct YawTag;
// 头部
#[derive(Debug, Component)]
pub struct HeadTag;
// 相机
#[derive(Debug, Component)]
pub struct CameraTag;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ControllerSet {
    INPUT_TO_EVENT,
    INPUT_TO_LOOK,
    FORWARD_UP,
}

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // 注册事件
        app.add_event::<PitchEvent>()
            .add_event::<YawEvent>()
            .add_event::<LookEvent>()
            .add_event::<LookDeltaEvent>()
            .add_event::<TranslationEvent>()
            .add_event::<ImpulseEvent>()
            .add_event::<ForceEvent>()
            .init_resource::<MouseSettings>()
            .add_systems(Startup, initial_grab_cursor)
            .configure_sets(
                PreUpdate,
                // chain() will ensure sets run in the order they are listed
                (
                    ControllerSet::INPUT_TO_EVENT,
                    ControllerSet::INPUT_TO_LOOK,
                    ControllerSet::FORWARD_UP,
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                (
                    cursor_grab,
                    (input_to_events).in_set(ControllerSet::INPUT_TO_EVENT),
                    (input_to_look)
                        .in_set(ControllerSet::INPUT_TO_LOOK)
                        .after(ControllerSet::INPUT_TO_EVENT),
                    (forward_up)
                        .in_set(ControllerSet::FORWARD_UP)
                        .after(ControllerSet::INPUT_TO_EVENT)
                        .after(ControllerSet::INPUT_TO_LOOK),
                ),
            );
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub run: bool,
    pub jump: bool,
    pub up: bool,
    pub down: bool,
}

#[derive(Debug, Component, Clone, Copy)]
pub struct CharacterController {
    pub input_map: InputMap,
    pub fly: bool,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub jump_speed: f32,
    pub velocity: Vec3,
    pub jumping: bool,
    pub dt: f32,
    pub sim_to_render: f32,
    pub input_state: InputState,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            input_map: InputMap::default(),
            fly: false,
            walk_speed: 5.0,
            run_speed: 8.0,
            jump_speed: 6.0,
            velocity: Vec3::ZERO,
            jumping: false,
            dt: 1.0 / 60.0,
            sim_to_render: 0.0,
            input_state: InputState::default(),
        }
    }
}

#[derive(Debug, Component)]
pub struct Mass {
    pub mass: f32,
}

impl Mass {
    pub fn new(mass: f32) -> Self {
        Self { mass }
    }
}

pub fn input_to_events(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut translation_events: EventWriter<TranslationEvent>,
    mut impulse_events: EventWriter<ImpulseEvent>,
    mut force_events: EventWriter<ForceEvent>,
    mut controller_query: Query<(&Mass, &LookEntity, &mut CharacterController)>,
    look_direction_query: Query<&LookDirection>,
) {
    let xz = Vec3::new(1.0, 0.0, 1.0);
    for (mass, look_entity, mut controller) in controller_query.iter_mut() {
        controller.sim_to_render += time.delta_seconds();

        if keyboard_input.just_pressed(controller.input_map.key_fly) {
            controller.fly = !controller.fly;
        }
        if keyboard_input.pressed(controller.input_map.key_forward) {
            controller.input_state.forward = true;
        }
        if keyboard_input.pressed(controller.input_map.key_backward) {
            controller.input_state.backward = true;
        }
        if keyboard_input.pressed(controller.input_map.key_right) {
            controller.input_state.right = true;
        }
        if keyboard_input.pressed(controller.input_map.key_left) {
            controller.input_state.left = true;
        }
        if keyboard_input.pressed(controller.input_map.key_run) {
            controller.input_state.run = true;
        }
        if keyboard_input.just_pressed(controller.input_map.key_jump) {
            controller.input_state.jump = true;
        }
        if keyboard_input.pressed(controller.input_map.key_fly_up) {
            controller.input_state.up = true;
        }
        if keyboard_input.pressed(controller.input_map.key_fly_down) {
            controller.input_state.down = true;
        }

        if controller.sim_to_render < controller.dt {
            continue;
        }
        // Calculate the remaining simulation to render time after all
        // simulation steps were taken
        controller.sim_to_render %= controller.dt;

        let look = look_direction_query
            .get_component::<LookDirection>(look_entity.0)
            .expect("Failed to get LookDirection from Entity");

        // Calculate forward / right / up vectors
        let (forward, right, up) = if controller.fly {
            (look.forward, look.right, look.up)
        } else {
            (
                (look.forward * xz).normalize(),
                (look.right * xz).normalize(),
                Vec3::Y,
            )
        };

        // Calculate the desired velocity based on input
        let mut desired_velocity = Vec3::ZERO;
        if controller.input_state.forward {
            desired_velocity += forward;
        }
        if controller.input_state.backward {
            desired_velocity -= forward;
        }
        if controller.input_state.right {
            desired_velocity += right;
        }
        if controller.input_state.left {
            desired_velocity -= right;
        }
        if controller.input_state.up {
            desired_velocity += up;
        }
        if controller.input_state.down {
            desired_velocity -= up;
        }

        // Limit x/z velocity to walk/run speed
        let speed = if controller.input_state.run {
            controller.run_speed
        } else {
            controller.walk_speed
        };
        desired_velocity = if desired_velocity.length_squared() > 1E-6 {
            desired_velocity.normalize() * speed
        } else {
            // No input - apply damping to the x/z of the current velocity
            controller.velocity * 0.5 * xz
        };

        // Handle jumping
        let was_jumping = controller.jumping;
        if !controller.fly {
            desired_velocity.y = if controller.input_state.jump {
                controller.jumping = true;
                controller.jump_speed
            } else {
                0.0
            };
        }

        // Calculate impulse - the desired momentum change for the time period
        let delta_velocity =
            desired_velocity - controller.velocity * if controller.fly { Vec3::ONE } else { xz };
        let impulse = delta_velocity * mass.mass;
        if impulse.length_squared() > 1E-6 {
            impulse_events.send(ImpulseEvent::new(&impulse));
        }

        // Calculate force - the desired rate of change of momentum for the time period
        let force = impulse / controller.dt;
        if force.length_squared() > 1E-6 {
            force_events.send(ForceEvent::new(&force));
        }

        controller.velocity.x = desired_velocity.x;
        controller.velocity.z = desired_velocity.z;
        controller.velocity.y = if !controller.fly && was_jumping {
            // Apply gravity for kinematic simulation
            (-9.81f32).mul_add(controller.dt, controller.velocity.y)
        } else {
            desired_velocity.y
        };

        let translation = controller.velocity * controller.dt;
        if translation.length_squared() > 1E-6 {
            translation_events.send(TranslationEvent::new(&translation));
        }

        controller.input_state = InputState::default();
    }
}

pub fn controller_to_yaw(
    mut yaws: EventReader<YawEvent>,
    mut query: Query<&mut Transform, With<YawTag>>,
) {
    if let Some(yaw) = yaws.iter().next() {
        for mut transform in query.iter_mut() {
            transform.rotation = Quat::from_rotation_y(**yaw);
        }
    }
}

pub fn controller_to_pitch(
    mut pitches: EventReader<PitchEvent>,
    mut query: Query<&mut Transform, With<HeadTag>>,
) {
    if let Some(pitch) = pitches.iter().next() {
        for mut transform in query.iter_mut() {
            transform.rotation = Quat::from_rotation_x(**pitch);
        }
    }
}

// 初始化光标
fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

// 光标显示或者隐藏系统
fn cursor_grab(
    keys: Res<Input<KeyCode>>,
    controller_query: Query<(&CharacterController)>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let controller = controller_query.single();
    let input_map = controller.input_map;
    if let Ok(mut window) = primary_window.get_single_mut() {
        if keys.just_pressed(input_map.toggle_grab_cursor) {
            toggle_grab_cursor(&mut window);
        }
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}
