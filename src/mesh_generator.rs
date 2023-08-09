use std::collections::HashSet;

use bevy::{
    prelude::{
        AlphaMode, Assets, Color, Commands, Entity, Handle, MaterialMeshBundle, Mesh, PbrBundle,
        Res, ResMut, Resource, StandardMaterial, SystemSet, Transform,
    },
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_rapier3d::{
    prelude::{Collider, RigidBody},
    rapier::prelude::{RigidBodyType, SharedShape},
};
use bincode::de;

use crate::{
    chunk::{find_chunk_keys_array_by_shpere_y_0, ChunkKey, NeighbourOffest},
    chunk_generator::ChunkMap,
    clip_spheres::ClipSpheres,
    mesh::{gen_mesh, gen_mesh_water, pick_water},
    mesh_material::MaterialStorge,
    voxel::Voxel,
    voxel_config::MaterailConfiguration,
    SmallKeyHashMap, CHUNK_SIZE, VIEW_RADIUS,
};

#[derive(Debug, Clone, Resource, Default)]
pub struct MeshManager {
    pub mesh_storge: SmallKeyHashMap<ChunkKey, Handle<Mesh>>,
    pub entities: SmallKeyHashMap<ChunkKey, Entity>,
    pub water_entities: SmallKeyHashMap<ChunkKey, Entity>,
    pub fast_key: HashSet<ChunkKey>,
}

#[derive(Resource)]
pub struct MeshTasks {
    pub tasks: Vec<Task<(Vec<Voxel>, ChunkKey)>>,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MeshSystem {
    UPDATE_MESH,
}

pub fn gen_mesh_system(
    mut chunk_map: ResMut<ChunkMap>,
    mut mesh_manager: ResMut<MeshManager>,
    clip_spheres: Res<ClipSpheres>,
    neighbour_offest: Res<NeighbourOffest>,
    mut mesh_task: ResMut<MeshTasks>,
) {
    let pool = AsyncComputeTaskPool::get();
    for key in
        find_chunk_keys_array_by_shpere_y_0(clip_spheres.new_sphere, neighbour_offest.0.clone())
            .drain(..)
    {
        if !mesh_manager.entities.contains_key(&key) && !mesh_manager.fast_key.contains(&key) {
            if (!chunk_map.map_data.contains_key(&key)) {
                // 这里没有加载好地图数据前 先不加载数据
                return;
            }
            // 无论如何都插入进去 放置下次重复检查
            mesh_manager.fast_key.insert(key);
            let volexs: Vec<Voxel> = chunk_map.get_with_neighbor_full_y(key);
            let task = pool.spawn(async move { (volexs.clone(), key.clone()) });
            mesh_task.tasks.push(task);
        }
    }
}

pub fn update_mesh_system(
    mut commands: Commands,
    mut mesh_manager: ResMut<MeshManager>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut mesh_task: ResMut<MeshTasks>,
    materials: Res<MaterialStorge>,
    material_config: Res<MaterailConfiguration>,
    mut materials_assets: ResMut<Assets<StandardMaterial>>,
) {
    let l = mesh_task.tasks.len().min(3);
    for ele in mesh_task.tasks.drain(..l) {
        match futures_lite::future::block_on(futures_lite::future::poll_once(ele)) {
            Some((voxels, chunk_key)) => {
                if mesh_manager.entities.contains_key(&chunk_key) {
                    return;
                } else {
                    match gen_mesh(voxels.to_owned(), material_config.clone()) {
                        Some(render_mesh) => {
                            let mesh_handle = mesh_assets.add(render_mesh);
                            mesh_manager
                                .mesh_storge
                                .insert(chunk_key, mesh_handle.clone());
                            mesh_manager.entities.insert(
                                chunk_key,
                                commands
                                    .spawn(MaterialMeshBundle {
                                        transform: Transform::from_xyz(
                                            (chunk_key.0.x * CHUNK_SIZE) as f32
                                                - CHUNK_SIZE as f32 / 2.0
                                                - 1.0,
                                            -128.0 + CHUNK_SIZE as f32 / 2.0,
                                            (chunk_key.0.z * CHUNK_SIZE) as f32
                                                - CHUNK_SIZE as f32 / 2.0
                                                - 1.0,
                                        ),
                                        mesh: mesh_handle.clone(),
                                        material: materials.0.clone(),
                                        ..Default::default()
                                    })
                                    .id(),
                            );
                        }
                        None => {}
                    };
                    match gen_mesh_water(pick_water(voxels.clone()), material_config.clone()) {
                        Some(water_mesh) => {
                            mesh_manager.water_entities.insert(
                                chunk_key,
                                commands
                                    .spawn(MaterialMeshBundle {
                                        transform: Transform::from_xyz(
                                            (chunk_key.0.x * CHUNK_SIZE) as f32
                                                - CHUNK_SIZE as f32 / 2.0
                                                - 1.0,
                                            -128.0 + CHUNK_SIZE as f32 / 2.0,
                                            (chunk_key.0.z * CHUNK_SIZE) as f32
                                                - CHUNK_SIZE as f32 / 2.0
                                                - 1.0,
                                        ),
                                        mesh: mesh_assets.add(water_mesh),
                                        material: materials_assets.add(StandardMaterial {
                                            base_color: Color::rgba(
                                                10. / 255.,
                                                18. / 255.,
                                                246. / 255.,
                                                0.6,
                                            ),
                                            alpha_mode: AlphaMode::Blend,
                                            ..Default::default()
                                        }),
                                        ..Default::default()
                                    })
                                    .id(),
                            );
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }
    }
}

pub fn deleter_mesh_system(
    mut commands: Commands,
    mut mesh_manager: ResMut<MeshManager>,
    neighbour_offest: Res<NeighbourOffest>,
    clip_spheres: Res<ClipSpheres>,
) {
    let mut chunks_to_remove = HashSet::new();
    for key in
        find_chunk_keys_array_by_shpere_y_0(clip_spheres.old_sphere, neighbour_offest.0.clone())
            .drain(..)
    {
        chunks_to_remove.insert(key);
    }

    for key in
        find_chunk_keys_array_by_shpere_y_0(clip_spheres.new_sphere, neighbour_offest.0.clone())
            .drain(..)
    {
        chunks_to_remove.remove(&key);
    }

    for chunk_key in chunks_to_remove.into_iter() {
        if let Some(entity) = mesh_manager.entities.remove(&chunk_key) {
            mesh_manager.fast_key.remove(&chunk_key);
            commands.entity(entity).despawn();
        }
        if let Some(entity) = mesh_manager.water_entities.remove(&chunk_key) {
            mesh_manager.fast_key.remove(&chunk_key);
            commands.entity(entity).despawn();
        }
    }
}
