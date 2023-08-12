// 测试使用命令

use bevy::prelude::{
    App, Component, EventReader, IntoSystemConfigs, Plugin, PreUpdate, Query, Res, ResMut,
    Transform, Update, Vec3, With,
};
use bevy_console::{
    AddConsoleCommand, ConsoleCommand, ConsoleCommandEntered, ConsoleOpen, ConsolePlugin,
    ConsoleSet,
};
use clap::Parser;
use controller::controller::ControllerFlag;

use crate::{
    clip_spheres::{ClipSpheres, Sphere3},
    player_controller::PlayerMe,
    player_ui::ToolbarContent,
    staff::StaffInfoStroge,
    VIEW_RADIUS,
};

pub struct ConsoleCommandPlugins;

impl Plugin for ConsoleCommandPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConsolePlugin)
            .add_systems(PreUpdate, sync_flags)
            .add_systems(Update, raw_commands.in_set(ConsoleSet::Commands))
            .add_console_command::<TpCommand, _>(tp_commands::<PlayerMe>)
            .add_console_command::<LoadToolbarCommand, _>(load_toolbar_commands);
    }
}

// 保持打开时不能操作人物
fn sync_flags(mut controller_flag: ResMut<ControllerFlag>, console_open: Res<ConsoleOpen>) {
    controller_flag.flag = !console_open.open;
}

fn raw_commands(mut console_commands: EventReader<ConsoleCommandEntered>) {
    for ConsoleCommandEntered { command_name, args } in console_commands.iter() {
        println!(r#"Entered command "{command_name}" with args {:#?}"#, args);
    }
}

// transform body to point
#[derive(Parser, ConsoleCommand)]
#[command(name = "tp", about = "transform body to point")]
struct TpCommand {
    /// x
    x: Option<f32>,
    /// x
    y: Option<f32>,
    /// x
    z: Option<f32>,
}

fn tp_commands<T>(
    mut log: ConsoleCommand<TpCommand>,
    mut clip_spheres: ResMut<ClipSpheres>,
    mut query: Query<&mut Transform, With<T>>,
) where
    T: Component,
{
    if let Some(Ok(TpCommand { x, y, z })) = log.take() {
        if let Some(xx) = x {
            if let Some(yy) = y {
                if let Some(zz) = z {
                    clip_spheres.old_sphere = clip_spheres.new_sphere;
                    clip_spheres.new_sphere = Sphere3 {
                        center: Vec3::new(xx, yy, zz),
                        radius: VIEW_RADIUS,
                    };
                    let mut tf = query.get_single_mut().unwrap();
                    tf.translation = Vec3::new(xx, yy, zz);
                    log.ok();
                } else {
                    log.failed();
                }
            } else {
                log.failed();
            }
        } else {
            log.failed();
        }
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "load_toolbar", about = "load default tool_bar to test")]
pub struct LoadToolbarCommand;

pub fn load_toolbar_commands(
    mut load_bar: ConsoleCommand<LoadToolbarCommand>,
    staff_infos: Res<StaffInfoStroge>,
    mut query: Query<&mut ToolbarContent>,
) {
    if let Some(Ok(LoadToolbarCommand)) = load_bar.take() {
        for mut tool_bar in &mut query {
            if let Some(staff) = staff_infos.data.get(&tool_bar.index) {
                tool_bar.staff = Some(staff.clone());
            }
        }
        load_bar.ok();
    }
}
