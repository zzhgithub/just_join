use crate::classes::*;
use bevy::{
    input::mouse::MouseWheel,
    prelude::{
        App, AssetServer, Changed, Color, Commands, Component, EventReader, Events, Input, KeyCode,
        Plugin, Query, Res, ResMut, Resource, Startup, Update,
    },
    ui::{BorderColor, Interaction, Node},
};
use bevy_ui_dsl::*;
use controller::controller::ControllerFlag;

#[macro_export]
macro_rules! add_keyboard_toolbar {
    ($key: expr,$value: expr,$class: expr,$change:expr) => {
        if $class.just_pressed($key) {
            $change.index = $value;
        }
    };
}

// 用户操作相关的UI
pub struct PlayerUiPlugin;

impl Plugin for PlayerUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActiveToolbar { index: 0 })
            .add_systems(Startup, setup_ui)
            .add_systems(Update, active_system)
            .add_systems(Update, (choose_toolbar, choose_by_wheel, choose_by_key));
    }
}

pub fn setup_ui(mut commands: Commands, assets: Res<AssetServer>) {
    root(c_root, &assets, &mut commands, |p| {
        node(c_buttom, p, |p| {
            // todo 上面是血条？这里需要什么属性？
            text("这里是用来测试字体显示的!", c_text, c_pixel, p);
            // 这里是 node 里面是横线排列的
            grid(1, 9, c_grid, p, |p, _row, _col| {
                // 这里是一个节点 里面 有一个图片
                buttoni(c_toolbar_box_normal, Toolbar { index: _col }, p, |p| {
                    node(c_overflow, p, |p| {
                        image(c_inv_slot, p);
                        // 测试两个图片的显示
                        image(c_test_staff, p);
                    });
                });
            });
        });
    });
}

// 测试和选中状态是否相符

#[derive(Resource)]
pub struct ActiveToolbar {
    pub index: usize,
}

#[derive(Debug, Component)]
pub struct Toolbar {
    pub index: usize,
}

fn active_system(
    mut query: Query<(&mut Node, &mut BorderColor, &Toolbar)>,
    active_toolbar: Res<ActiveToolbar>,
) {
    for (mut node, mut color, tool_bar) in &mut query {
        if tool_bar.index == active_toolbar.index {
            *color = Color::RED.into();
        } else {
            *color = Color::rgba(0., 0., 0., 0.).into();
        }
    }
}

fn choose_by_key(
    controller_flag: Res<ControllerFlag>,
    keyboard_input: Res<Input<KeyCode>>,
    mut active_toolbar: ResMut<ActiveToolbar>,
) {
    // 如果是不能控制状态禁止控制
    if (!controller_flag.flag) {
        return;
    }
    add_keyboard_toolbar!(KeyCode::Key1, 0, keyboard_input, active_toolbar);
    add_keyboard_toolbar!(KeyCode::Key2, 1, keyboard_input, active_toolbar);
    add_keyboard_toolbar!(KeyCode::Key3, 2, keyboard_input, active_toolbar);
    add_keyboard_toolbar!(KeyCode::Key4, 3, keyboard_input, active_toolbar);
    add_keyboard_toolbar!(KeyCode::Key5, 4, keyboard_input, active_toolbar);
    add_keyboard_toolbar!(KeyCode::Key6, 5, keyboard_input, active_toolbar);
    add_keyboard_toolbar!(KeyCode::Key7, 6, keyboard_input, active_toolbar);
    add_keyboard_toolbar!(KeyCode::Key8, 7, keyboard_input, active_toolbar);
    add_keyboard_toolbar!(KeyCode::Key9, 8, keyboard_input, active_toolbar);
}

fn choose_by_wheel(
    mut active_toolbar: ResMut<ActiveToolbar>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    for event in mouse_wheel_events.iter() {
        // println!("{:?}", event);
        if event.y > 0. {
            active_toolbar.index += 1;
        } else {
            if active_toolbar.index == 0 {
                active_toolbar.index = 8;
            } else {
                active_toolbar.index -= 1;
            }
        }
        if active_toolbar.index > 8 {
            active_toolbar.index = 0;
        }
    }
}

fn choose_toolbar(
    ui_entities: Query<(&Toolbar, &Interaction), Changed<Interaction>>,
    mut active_toolbar: ResMut<ActiveToolbar>,
) {
    for (toolbar, inter) in &ui_entities {
        match (toolbar, inter) {
            (Toolbar { index }, Interaction::Pressed) => {
                active_toolbar.index = index.clone();
            }
            _ => {}
        }
    }
}
