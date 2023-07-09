// 角色相关
use bevy::ecs::event::ManualEventReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::{
    warn, Added, App, AssetServer, Camera3dBundle, Commands, Component, Entity,
    EnvironmentMapLight, EulerRot, Events, Input, IntoSystemAppConfig, KeyCode, Plugin, Quat,
    Query, Res, ResMut, Resource, Transform, Vec2, Vec3, With, World,
};
use bevy::time::Time;
use bevy::window::{CursorGrabMode, PrimaryWindow, Window};
use bevy_atmosphere::prelude::AtmosphereCamera;
use bevy_egui::egui::epaint::Shadow;
use bevy_egui::egui::{self, Color32, Pos2, Stroke};
use bevy_egui::EguiContext;
use bevy_rapier3d::prelude::{Collider, LockedAxes};
use bevy_rapier3d::prelude::{QueryFilter, RigidBody};

#[derive(Debug, Component)]
pub struct PlayerController;

// Copy Code by FlyCam!!!!

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Resource, Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

/// Mouse sensitivity and movement speed
#[derive(Resource)]
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 12.,
        }
    }
}

/// Key configuration
#[derive(Resource)]
pub struct KeyBindings {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    // todo 后续这里要修改
    pub move_ascend: KeyCode,
    pub move_descend: KeyCode,
    pub toggle_grab_cursor: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::W,
            move_backward: KeyCode::S,
            move_left: KeyCode::A,
            move_right: KeyCode::D,
            move_ascend: KeyCode::Space,
            move_descend: KeyCode::LShift,
            toggle_grab_cursor: KeyCode::Escape,
        }
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

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    key_bindings: Res<KeyBindings>,
    mut query: Query<(&PlayerController, &mut Transform)>, //    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    if let Ok(window) = primary_window.get_single() {
        for (_camera, mut transform) in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        let key = *key;
                        if key == key_bindings.move_forward {
                            velocity += forward;
                        } else if key == key_bindings.move_backward {
                            velocity -= forward;
                        } else if key == key_bindings.move_left {
                            velocity -= right;
                        } else if key == key_bindings.move_right {
                            velocity += right;
                        } else if key == key_bindings.move_ascend {
                            velocity += Vec3::Y;
                        } else if key == key_bindings.move_descend {
                            velocity -= Vec3::Y;
                        }
                    }
                }

                velocity = velocity.normalize_or_zero();

                transform.translation += velocity * time.delta_seconds() * settings.speed
            }
        }
    } else {
        warn!("Primary window not found for `player_move`!");
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, With<PlayerController>>,
) {
    if let Ok(window) = primary_window.get_single() {
        for mut transform in query.iter_mut() {
            for ev in state.reader_motion.iter(&motion) {
                let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                        let window_scale = window.height().min(window.width());
                        pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                        yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                    }
                }

                pitch = pitch.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

fn cursor_grab(
    keys: Res<Input<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        if keys.just_pressed(key_bindings.toggle_grab_cursor) {
            toggle_grab_cursor(&mut window);
        }
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}

fn initial_grab_on_flycam_spawn(
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    query_added: Query<Entity, Added<PlayerController>>,
) {
    if query_added.is_empty() {
        return;
    }

    if let Ok(window) = &mut primary_window.get_single_mut() {
        toggle_grab_cursor(window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

/// Contains everything needed to add first-person fly camera behavior to your game
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .init_resource::<KeyBindings>()
            .add_system(setup_player.on_startup())
            .add_system(initial_grab_cursor.on_startup())
            .add_system(player_move)
            .add_system(player_look)
            .add_system(cursor_grab)
            .add_system(egui_center_cursor_system);
    }
}

#[derive(Resource)]
pub struct PlayerStorge(pub Entity);

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let palyer = commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(-2.0, 20.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..Default::default()
            },
            EnvironmentMapLight {
                diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
                specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            },
            AtmosphereCamera::default(),
            PlayerController,
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED_X
                | LockedAxes::ROTATION_LOCKED_Y
                | LockedAxes::ROTATION_LOCKED_Z,
            // 这里尝试插入胶囊 对于需要的prb显示 后续才进行
            // 这里的人物模型扩大呢？
            Collider::ball(1.0),
        ))
        .id();
    commands.insert_resource(PlayerStorge(palyer));
}

// 添加中心十字
fn egui_center_cursor_system(
    mut egui_context_q: Query<&mut EguiContext, With<PrimaryWindow>>,
    window_qurey: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(egui_context) = egui_context_q.get_single_mut() else{return;};
    let mut egui_context = egui_context.clone();

    let Ok(window) = window_qurey.get_single() else{return;};
    let size = Vec2::new(window.width(), window.height());
    // 透明的屏幕！
    let my_frame = egui::containers::Frame {
        inner_margin: egui::style::Margin {
            left: 10.,
            right: 10.,
            top: 10.,
            bottom: 10.,
        },
        outer_margin: egui::style::Margin {
            left: 10.,
            right: 10.,
            top: 10.,
            bottom: 10.,
        },
        rounding: egui::Rounding {
            nw: 1.0,
            ne: 1.0,
            sw: 1.0,
            se: 1.0,
        },
        shadow: Shadow {
            extrusion: 1.0,
            color: Color32::TRANSPARENT,
        },
        fill: Color32::TRANSPARENT,
        stroke: egui::Stroke::new(2.0, Color32::TRANSPARENT),
    };

    egui::CentralPanel::default()
        .frame(my_frame)
        .show(&egui_context.get_mut(), |ui| {
            //  = Color32::TRANSPARENT;

            // let size = ui.available_size();
            // 计算十字准星的位置和大小
            let crosshair_size = 20.0;
            let crosshair_pos = egui::Pos2::new(
                size.x / 2.0 - crosshair_size / 2.0,
                size.y / 2.0 - crosshair_size / 2.0,
            );
            // 外边框
            let crosshair_rect =
                egui::Rect::from_min_size(crosshair_pos, egui::Vec2::splat(crosshair_size));

            // 绘制十字准星的竖线
            let line_width = 2.0;
            let line_rect = egui::Rect::from_min_max(
                egui::Pos2::new(
                    crosshair_rect.center().x - line_width / 2.0,
                    crosshair_rect.min.y,
                ),
                egui::Pos2::new(
                    crosshair_rect.center().x + line_width / 2.0,
                    crosshair_rect.max.y,
                ),
            );
            ui.painter()
                .rect_filled(line_rect, 1.0, egui::Color32::WHITE);

            // 绘制十字准星的横线
            let line_rect = egui::Rect::from_min_max(
                egui::Pos2::new(
                    crosshair_rect.min.x,
                    crosshair_rect.center().y - line_width / 2.0,
                ),
                egui::Pos2::new(
                    crosshair_rect.max.x,
                    crosshair_rect.center().y + line_width / 2.0,
                ),
            );
            ui.painter()
                .rect_filled(line_rect, 1.0, egui::Color32::WHITE);

            // todo 这里也可以添加下方物品栏
        });
}
