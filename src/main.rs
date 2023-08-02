use bevy::{
    pbr::{wireframe::WireframePlugin, DirectionalLightShadowMap, Shadow},
    prelude::{
        bevy_main, AmbientLight, App, AssetServer, Assets, BuildChildren, Camera3d, Color,
        Commands, Component, FixedUpdate, MaterialPlugin, Msaa, ParamSet, PointLight,
        PointLightBundle, PostUpdate, PreUpdate, Query, Res, ResMut, Startup, SystemSet, Transform,
        Update, Vec3, With, Without,
    },
    render::render_resource::RenderPipeline,
    DefaultPlugins, reflect::FromReflect,
};
// use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_rapier3d::{
    prelude::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use chunk::generate_offset_resoure;
use chunk_generator::{chunk_generate_system, ChunkMap};
use clip_spheres::{update_clip_shpere_system, ClipSpheres, Sphere3};
use controller::controller::{CameraTag, HeadTag};
use inspector_egui::inspector_ui;
use map_database::MapDataBase;
use mesh_generator::{deleter_mesh_system, update_mesh_system, MeshManager, MeshTasks};

use bevy_egui::EguiPlugin;
use mesh_material::{BindlessMaterial, MaterialStorge};
use palyer::{PlayerController, PlayerPlugin};
use player_controller::{PlayerControllerPlugin, PlayerMe};
use ray_cast::MyRayCastPlugin;
use sky::SkyPlugin;
use structopt::StructOpt;
use voxel_config::VoxelMaterialToolPulgin;
// use sky::SkyPlugin;

mod chunk;
mod chunk_generator;
mod clip_spheres;
mod inspector_egui;
mod map_database;
mod map_generator;
mod mesh;
mod mesh_generator;
mod mesh_material;
mod palyer;
mod player_controller;
mod ray_cast;
mod sky;
mod voxel;
mod voxel_config;

pub type SmallKeyHashMap<K, V> = ahash::AHashMap<K, V>;

// const zone
pub const VIEW_RADIUS: f32 = 120.00;
pub const CHUNK_SIZE: i32 = 16;
// 贴图个数
pub const MAX_TEXTURE_COUNT: usize = 5;

#[derive(Debug, StructOpt)]
enum RunMode {
    Tool,
    Game,
}

#[bevy_main]
fn main() {
    let mut app_builder = App::new();

    match RunMode::from_args() {
        RunMode::Tool => {
            app_builder
                .add_plugins(DefaultPlugins)
                .add_plugins(MaterialPlugin::<BindlessMaterial>::default())
                .add_plugins(EguiPlugin)
                .add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin)
                .add_plugins(VoxelMaterialToolPulgin)
                .run();
        }
        RunMode::Game => {
            app_builder
                .add_plugins(DefaultPlugins)
                .add_plugins(MaterialPlugin::<BindlessMaterial>::default())
                // .add_plugins(PlayerPlugin)
                //FIXME: 这个物品获取又基本无效了 物理引擎不能识别到碰撞了
                // .add_plugins(MyRayCastPlugin)
                // 0.11.0 不能使用了
                .add_plugins(SkyPlugin)
                .add_plugins(EguiPlugin)
                .add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin) // adds default options and `InspectorEguiImpl`s
                .add_system(inspector_ui)
                .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
                .add_plugins(PlayerControllerPlugin)
                // .add_plugin(RapierDebugRenderPlugin::default())
                .insert_resource(Msaa::Sample4)
                // 这里是设置了UI
                .add_systems(Startup, setup)
                .add_systems(PreUpdate, chunk_generate_system)
                .add_systems(PreUpdate, update_clip_shpere_system::<PlayerMe>)
                .add_systems(PostUpdate, deleter_mesh_system)
                .add_systems(FixedUpdate, update_mesh_system)
                // 测试时使用的光源跟随
                .add_systems(Update, light_follow_camera_system::<HeadTag>)
                .run();
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
struct ChunkFlush;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<BindlessMaterial>>,
) {
    // init resource of clip Spheres
    let eye = Vec3::ZERO;
    let init_shpere = Sphere3 {
        center: eye,
        radius: VIEW_RADIUS,
    };

    let clip_spheres = ClipSpheres {
        old_sphere: init_shpere,
        new_sphere: init_shpere,
    };
    commands.insert_resource(clip_spheres);

    // init resource of chunkKey offset
    commands.insert_resource(generate_offset_resoure(VIEW_RADIUS));

    // init chunkMap
    commands.insert_resource(ChunkMap::new());

    // init MeshManager
    commands.insert_resource(MeshManager::default());

    // init MeshTasks
    commands.insert_resource(MeshTasks { tasks: Vec::new() });

    // init MapData
    let mut db = MapDataBase::new("world_test");
    // db.test_gen();

    commands.insert_resource(db);

    // 加载材质图案
    commands.insert_resource(MaterialStorge::init(asset_server, materials));

    // 添加氛围光
    // commands.insert_resource(AmbientLight {
    //     color: Color::WHITE,
    //     brightness: 0.0,
    // });

    // commands.insert_resource(DirectionalLightShadowMap { size: 4096 });
    // FIXME: 设置光源 有天空盒子不需要设置光源测试了
    commands.spawn(
        (PointLightBundle {
            point_light: PointLight {
                intensity: 10000.0,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
            ..Default::default()
        }),
    );
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
