use bevy::{
    prelude::{bevy_main, App, Commands, PointLightBundle, Transform, Vec3},
    DefaultPlugins,
};
use bevy_flycam::{FlyCam, PlayerPlugin};
use chunk::generate_offset_resoure;
use chunk_generator::{chunk_generate_system, ChunkMap};
use clip_spheres::{update_clip_shpere_system, ClipSpheres, Sphere3};
use mesh_generator::{deleter_mesh_system, update_mesh_system, MeshManager};

mod chunk;
mod chunk_generator;
mod clip_spheres;
mod mesh;
mod mesh_generator;
mod voxel;

pub type SmallKeyHashMap<K, V> = ahash::AHashMap<K, V>;

// const zone
pub const VIEW_RADIUS: f32 = 80.00;
pub const CHUNK_SIZE: i32 = 16;

#[bevy_main]
fn main() {
    let mut app_builder = App::new();

    app_builder
        .add_startup_system(setup)
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_system(update_clip_shpere_system::<FlyCam>)
        .add_system(chunk_generate_system)
        // 这个会使性能严重下降
        // .add_system(deleter_mesh_system)
        .add_system(update_mesh_system)
        .run();
}

fn setup(mut commands: Commands) {
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

    // 设置光源
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 500.0, 0.0)),
        ..Default::default()
    });
}