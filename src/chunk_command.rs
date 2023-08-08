// chunk 修改命令相关

use bevy::{
    prelude::{Commands, Input, Last, MouseButton, Plugin, ResMut, Resource, Update, Vec3},
    tasks::{AsyncComputeTaskPool, Task},
};
use ndshape::{ConstShape, ConstShape3u32};

use crate::{
    chunk::{get_chunk_key_i3_by_vec3, ChunkKey},
    chunk_generator::ChunkMap,
    collider_generator::ColliderManager,
    mesh_generator::MeshManager,
    ray_cast::ChooseCube,
    voxel::Voxel,
    CHUNK_SIZE, CHUNK_SIZE_U32,
};

#[derive(Debug)]
pub enum ChunkCommands {
    Change {
        chunk_key: ChunkKey,
        pos: [u32; 3],
        voxel_type: Voxel,
    },
}

#[derive(Debug, Resource)]
pub struct ChunkCommandsTasks {
    pub tasks: Vec<Task<ChunkCommands>>,
}

// 处理更新请求
pub fn do_command_tasks(
    mut commands: Commands,
    mut chunk_map: ResMut<ChunkMap>,
    mut tasks: ResMut<ChunkCommandsTasks>,
    mut collider_manager: ResMut<ColliderManager>,
    mut mesh_manager: ResMut<MeshManager>,
) {
    //FIXME: 首先先分组 同一个chunkMap的数据一起处理合并 后面处理多了再说
    for ele in tasks.tasks.drain(..) {
        match futures_lite::future::block_on(futures_lite::future::poll_once(ele)) {
            Some(chunk_command) => {
                match chunk_command {
                    ChunkCommands::Change {
                        chunk_key,
                        pos,
                        voxel_type,
                    } => {
                        // 第一步找到地图上的 chunk 数据修改
                        match chunk_map.map_data.get_mut(&chunk_key) {
                            Some(voxel) => {
                                type SampleShape =
                                    ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
                                let index = SampleShape::linearize(pos) as usize;
                                voxel[index] = voxel_type;
                                let mut chunk_key_y0 = chunk_key.clone();
                                chunk_key_y0.0.y = 0;

                                if let Some(entity) = mesh_manager.entities.remove(&chunk_key_y0) {
                                    commands.entity(entity).despawn();
                                    mesh_manager.fast_key.remove(&chunk_key_y0);
                                }
                                if let Some(entity) =
                                    mesh_manager.water_entities.remove(&chunk_key_y0)
                                {
                                    commands.entity(entity).despawn();
                                    mesh_manager.fast_key.remove(&chunk_key_y0);
                                }
                                if let Some(entity) =
                                    collider_manager.entities.remove(&chunk_key_y0)
                                {
                                    commands.entity(entity).despawn();
                                }
                            }
                            None => {
                                println!("尝试修改没有生成chunk的数据");
                            }
                        }
                        // 然后 擅长 对应的chunk_y 层数据 对应的 chunk_mesh和 和 collider
                    }
                }
            }
            None => {}
        }
    }
}

pub fn build_or_break(
    mut mouse_button_input: ResMut<Input<MouseButton>>,
    mut choose_cube: ResMut<ChooseCube>,
    mut tasks: ResMut<ChunkCommandsTasks>,
) {
    let pool = AsyncComputeTaskPool::get();

    if mouse_button_input.just_pressed(MouseButton::Left) {
        // 这里按下了鼠标左键
        if let Some(pos) = choose_cube.center {
            // 点转成 chunk_key 和 x, y, z 的方法？
            let (chunk_key, xyz) = vec3_to_chunk_key_any_xyz(pos);
            println!("左键点击 要处理的方块是[{:?}][{:?}]", chunk_key, xyz);
            // TODO: 这生成一组数据
            // for x_new in 0..=15 {
            //     for y_new in 0..=15 {
            //         for z_new in 0..=15 {
            //             // let mut test = xyz.clone();
            //             // test[1] = y_new;
            //             let task = pool.spawn(async move {
            //                 ChunkCommands::Change {
            //                     chunk_key: chunk_key,
            //                     pos: [x_new, y_new, z_new],
            //                     voxel_type: Voxel::EMPTY,
            //                 }
            //             });
            //             tasks.tasks.push(task);
            //         }
            //     }
            // }
            let task = pool.spawn(async move {
                ChunkCommands::Change {
                    chunk_key: chunk_key,
                    pos: xyz,
                    voxel_type: Voxel::EMPTY,
                }
            });
            tasks.tasks.push(task);
        }
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        // 这里按下了鼠标右边键
        if let Some(pos) = choose_cube.out_center {
            // 点转成 chunk_key 和 x, y, z 的方法？
            let (chunk_key, xyz) = vec3_to_chunk_key_any_xyz(pos);
            println!("左键点击 要添加的方块是[{:?}][{:?}]", chunk_key, xyz);
            let task = pool.spawn(async move {
                ChunkCommands::Change {
                    chunk_key: chunk_key,
                    pos: xyz,
                    voxel_type: Voxel::FILLED,
                }
            });
            tasks.tasks.push(task);
        }
    }
}

pub fn vec3_to_chunk_key_any_xyz(pos: Vec3) -> (ChunkKey, [u32; 3]) {
    println!("此时的位置是: {:?}", pos);
    let chunk_key = ChunkKey(get_chunk_key_i3_by_vec3(pos));
    let x = (pos.x - (chunk_key.0.x * CHUNK_SIZE) as f32 + CHUNK_SIZE as f32 / 2. - 0.5) as u32;
    let y = (pos.y - (chunk_key.0.y * CHUNK_SIZE) as f32 + CHUNK_SIZE as f32 / 2. - 0.5) as u32;
    let z = (pos.z - (chunk_key.0.z * CHUNK_SIZE) as f32 + CHUNK_SIZE as f32 / 2. - 0.5) as u32;

    return (chunk_key, [x, y, z]);
}

pub struct ChunkCommandsPlugin;

impl Plugin for ChunkCommandsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // 初始化资源
        app.insert_resource(ChunkCommandsTasks { tasks: Vec::new() })
            .add_systems(Update, build_or_break)
            .add_systems(Last, do_command_tasks);
    }
}

#[test]
fn test_chunk_key() {
    let a = Vec3::new(1.0, 1.0, 1.0);
    let a_k = a.as_ivec3() / CHUNK_SIZE;

    let b = Vec3::new(-1.0, 1.0, 1.0);
    let b_k = b.as_ivec3() / CHUNK_SIZE;

    println!("a {:?} ", a_k);
    println!("b {:?} ", b_k);
}
