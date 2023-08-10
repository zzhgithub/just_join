// chunk 修改命令相关

use bevy::{
    prelude::{
        AlphaMode, Assets, Color, Commands, GlobalTransform, Input, Last, MaterialMeshBundle, Mesh,
        MouseButton, Plugin, Res, ResMut, Resource, StandardMaterial, Transform, Update, Vec3,
    },
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_rapier3d::prelude::{Collider, RigidBody};
use ndshape::{ConstShape, ConstShape3u32};

use crate::{
    chunk::{get_chunk_key_i3_by_vec3, ChunkKey},
    chunk_generator::ChunkMap,
    collider_generator::{ColliderManager, TerrainPhysics},
    mesh::{gen_mesh, gen_mesh_water, pick_water},
    mesh_generator::MeshManager,
    mesh_material::MaterialStorge,
    ray_cast::ChooseCube,
    voxel::Voxel,
    voxel_config::MaterailConfiguration,
    CHUNK_SIZE, CHUNK_SIZE_U32,
};

#[derive(Debug)]
pub enum ChunkCommands {
    Change {
        chunk_key: ChunkKey,
        pos: [u32; 3],
        voxel_type: Voxel,
    },
    UpdateMesh {
        chunk_key: ChunkKey,
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
    material_config: Res<MaterailConfiguration>,
    mut materials_assets: ResMut<Assets<StandardMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    materials: Res<MaterialStorge>,
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
                                // todo: 这里可以等重新生成结束后再去 擅长效果应该要好一点
                                // 要修改的数据
                                update_mesh(
                                    chunk_map.as_mut(),
                                    chunk_key_y0.clone(),
                                    material_config.clone(),
                                    mesh_manager.as_mut(),
                                    mesh_assets.as_mut(),
                                );
                                // 重新生成物理
                                // FIXME: 这里后续也不应该重新生成！
                                update_collider(
                                    &mut commands,
                                    mesh_manager.as_mut(),
                                    chunk_key_y0.clone(),
                                    mesh_assets.as_mut(),
                                    collider_manager.as_mut(),
                                );
                            }
                            None => {
                                println!("尝试修改没有生成chunk的数据");
                            }
                        }
                        // 然后 擅长 对应的chunk_y 层数据 对应的 chunk_mesh和 和 collider
                    }
                    ChunkCommands::UpdateMesh { chunk_key } => {
                        let mut chunk_key_y0 = chunk_key.clone();
                        chunk_key_y0.0.y = 0;
                        if (mesh_manager.mesh_storge.contains_key(&chunk_key_y0)) {
                            update_mesh(
                                chunk_map.as_mut(),
                                chunk_key_y0.clone(),
                                material_config.clone(),
                                mesh_manager.as_mut(),
                                mesh_assets.as_mut(),
                            );
                            update_collider(
                                &mut commands,
                                mesh_manager.as_mut(),
                                chunk_key_y0.clone(),
                                mesh_assets.as_mut(),
                                collider_manager.as_mut(),
                            );
                        }
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
            println!("左键点击 要打击的方块是[{:?}][{:?}]", chunk_key, xyz);
            let task = pool.spawn(async move {
                ChunkCommands::Change {
                    chunk_key: chunk_key,
                    pos: xyz,
                    voxel_type: Voxel::EMPTY,
                }
            });
            tasks.tasks.push(task);
            // FIXME: 遇到的边界问题 代码不优雅可以优化
            // 注意这里 有顺序问题!
            if xyz[0] == 0 {
                let mut new_chunk_key_i3 = chunk_key.0.clone();
                new_chunk_key_i3.x -= 1;
                let task_new = pool.spawn(async move {
                    ChunkCommands::UpdateMesh {
                        chunk_key: ChunkKey(new_chunk_key_i3),
                    }
                });
                tasks.tasks.push(task_new);
            }
            if xyz[0] == CHUNK_SIZE_U32 - 1 {
                let mut new_chunk_key_i3 = chunk_key.0.clone();
                new_chunk_key_i3.x += 1;
                let task_new = pool.spawn(async move {
                    ChunkCommands::UpdateMesh {
                        chunk_key: ChunkKey(new_chunk_key_i3),
                    }
                });
                tasks.tasks.push(task_new);
            }
            if xyz[2] == 0 {
                let mut new_chunk_key_i3 = chunk_key.0.clone();
                new_chunk_key_i3.z -= 1;
                let task_new = pool.spawn(async move {
                    ChunkCommands::UpdateMesh {
                        chunk_key: ChunkKey(new_chunk_key_i3),
                    }
                });
                tasks.tasks.push(task_new);
            }
            if xyz[2] == CHUNK_SIZE_U32 - 1 {
                let mut new_chunk_key_i3 = chunk_key.0.clone();
                new_chunk_key_i3.z += 1;
                let task_new = pool.spawn(async move {
                    ChunkCommands::UpdateMesh {
                        chunk_key: ChunkKey(new_chunk_key_i3),
                    }
                });
                tasks.tasks.push(task_new);
            }
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

pub fn update_collider(
    commands: &mut Commands,
    mesh_manager: &mut MeshManager,
    chunk_key_y0: ChunkKey,
    mesh_assets: &mut Assets<Mesh>,
    collider_manager: &mut ColliderManager,
) {
    if (mesh_manager.mesh_storge.contains_key(&chunk_key_y0)) {
        let mesh_handle = mesh_manager.mesh_storge.get(&chunk_key_y0).unwrap();
        if let Some(mesh) = mesh_assets.get(mesh_handle) {
            // 生成 collider 碰撞体
            if let Some(positions) = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
            {
                let indices: Vec<u32> = mesh.indices().unwrap().iter().map(|x| x as u32).collect();
                let collider_vertices: Vec<Vec3> =
                    positions.iter().cloned().map(|p| Vec3::from(p)).collect();
                let collider_indices: Vec<[u32; 3]> =
                    indices.chunks(3).map(|i| [i[0], i[1], i[2]]).collect();
                let collider: Collider = Collider::trimesh(collider_vertices, collider_indices);
                let entity = commands
                    .spawn((
                        TerrainPhysics,
                        Transform::from_xyz(
                            (chunk_key_y0.0.x * CHUNK_SIZE) as f32 - CHUNK_SIZE as f32 / 2.0 - 1.0,
                            -128.0 + CHUNK_SIZE as f32 / 2.0,
                            (chunk_key_y0.0.z * CHUNK_SIZE) as f32 - CHUNK_SIZE as f32 / 2.0 - 1.0,
                        ),
                        GlobalTransform::default(),
                    ))
                    .insert(RigidBody::Fixed)
                    .insert(collider)
                    .id();
                if let Some(old_id) = collider_manager.entities.insert(chunk_key_y0, entity) {
                    commands.entity(old_id).despawn();
                }
            }
        }
    }
}

pub fn update_mesh(
    chunk_map: &mut ChunkMap,
    chunk_key_y0: ChunkKey,
    material_config: MaterailConfiguration,
    mesh_manager: &mut MeshManager,
    mesh_assets: &mut Assets<Mesh>,
) {
    let volexs: Vec<Voxel> = chunk_map.get_with_neighbor_full_y(chunk_key_y0);
    match gen_mesh(volexs.to_owned(), material_config.clone()) {
        Some(render_mesh) => {
            let mesh_handle = mesh_manager.mesh_storge.get(&chunk_key_y0).unwrap();
            if let Some(mesh) = mesh_assets.get_mut(mesh_handle) {
                *mesh = render_mesh;
            }
            // 没有生成mesh就不管反正后面要生成
        }
        None => {}
    };
    match gen_mesh_water(pick_water(volexs.clone()), material_config.clone()) {
        Some(water_mesh) => {
            let mesh_handle = mesh_manager.water_mesh_storge.get(&chunk_key_y0).unwrap();
            if let Some(mesh) = mesh_assets.get_mut(mesh_handle) {
                *mesh = water_mesh;
            }
        }
        None => {}
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
    let (key, xyz) = vec3_to_chunk_key_any_xyz(Vec3::new(2.5, 24.5, 0.5));
    print!("{:?},{:?}", key, xyz);
}
