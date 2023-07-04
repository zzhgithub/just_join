use bevy::{
    prelude::{
        bevy_main, App, AssetServer, Assets, Commands, IntoSystemConfig, MaterialPlugin, ParamSet,
        PointLight, PointLightBundle, Query, Res, ResMut, SystemSet, Transform, Vec3, With,
        Without,
    },
    DefaultPlugins,
};
use bevy_flycam::{FlyCam, PlayerPlugin};
use chunk::generate_offset_resoure;
use chunk_generator::{chunk_generate_system, ChunkMap};
use clip_spheres::{update_clip_shpere_system, ClipSpheres, Sphere3};
use inspector_egui::inspector_ui;
use map_database::MapDataBase;
use mesh_generator::{deleter_mesh_system, update_mesh_system, MeshManager, MeshTasks};

use bevy_egui::EguiPlugin;
use mesh_material::{BindlessMaterial, MaterialStorge};

mod chunk;
mod chunk_generator;
mod clip_spheres;
mod inspector_egui;
mod map_database;
mod map_generator;
mod mesh;
mod mesh_generator;
mod mesh_material;
mod voxel;

pub type SmallKeyHashMap<K, V> = ahash::AHashMap<K, V>;

// const zone
pub const VIEW_RADIUS: f32 = 40.00;
pub const CHUNK_SIZE: i32 = 16;
// 贴图个数
pub const MAX_TEXTURE_COUNT: usize = 4;

#[bevy_main]
fn main() {
    let mut app_builder = App::new();

    app_builder
        .add_startup_system(setup)
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<BindlessMaterial>::default())
        .add_plugin(PlayerPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(bevy_inspector_egui::DefaultInspectorConfigPlugin) // adds default options and `InspectorEguiImpl`s
        .add_system(inspector_ui)
        .add_system(update_clip_shpere_system::<FlyCam>)
        .add_system(chunk_generate_system)
        .add_system(deleter_mesh_system)
        .add_system(update_mesh_system)
        // 测试时使用的光源跟随
        .add_system(light_follow_camera_system)
        .run();
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

    // 设置光源
    commands.spawn(
        (PointLightBundle {
            point_light: PointLight {
                intensity: 10000.0,
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
            ..Default::default()
        }),
    );
}

fn light_follow_camera_system(
    cam_q: Query<&Transform, With<FlyCam>>,
    mut light_q: Query<&mut Transform, (With<PointLight>, Without<FlyCam>)>,
) {
    let camera_position = if let Some(tfm) = cam_q.iter().next() {
        tfm.translation
    } else {
        return;
    };
    if let Some(mut tfm) = light_q.iter_mut().next() {
        tfm.translation = camera_position;
    }
}
