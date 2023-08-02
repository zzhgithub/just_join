use std::collections::HashSet;

use bevy::{
    prelude::{
        Assets, Color, Commands, Entity, MaterialMeshBundle, Mesh, PbrBundle, Res, ResMut,
        Resource, StandardMaterial, Transform,
    },
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_rapier3d::{
    prelude::{Collider, RigidBody},
    rapier::prelude::{RigidBodyType, SharedShape},
};

use crate::{
    chunk::{find_chunk_keys_array_by_shpere_y_0, ChunkKey, NeighbourOffest},
    chunk_generator::ChunkMap,
    clip_spheres::ClipSpheres,
    mesh::gen_mesh,
    mesh_material::MaterialStorge,
    voxel::Voxel,
    voxel_config::MaterailConfiguration,
    SmallKeyHashMap, CHUNK_SIZE, VIEW_RADIUS,
};

#[derive(Debug, Clone, Resource, Default)]
pub struct MeshManager {
    pub entities: SmallKeyHashMap<ChunkKey, Entity>,
    pub fast_key: HashSet<ChunkKey>,
}

#[derive(Resource)]
pub struct MeshTasks {
    pub tasks: Vec<Task<(ChunkKey, Mesh, Collider)>>,
}

pub fn update_mesh_system(
    mut commands: Commands,
    mut chunk_map: ResMut<ChunkMap>,
    mut mesh_manager: ResMut<MeshManager>,
    clip_spheres: Res<ClipSpheres>,
    neighbour_offest: Res<NeighbourOffest>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut mesh_task: ResMut<MeshTasks>,
    materials: Res<MaterialStorge>,
    material_config: Res<MaterailConfiguration>,
) {
    let pool = AsyncComputeTaskPool::get();
    for ele in mesh_task.tasks.drain(..) {
        match futures_lite::future::block_on(futures_lite::future::poll_once(ele)) {
            Some((chunk_key, mesh, collider)) => {
                mesh_manager.entities.insert(
                    chunk_key,
                    commands
                        .spawn(MaterialMeshBundle {
                            transform: Transform::from_xyz(
                                (chunk_key.0.x * CHUNK_SIZE) as f32,
                                -128.0,
                                (chunk_key.0.z * CHUNK_SIZE) as f32,
                            ),
                            mesh: mesh_assets.add(mesh),
                            material: materials.0.clone(),
                            ..Default::default()
                        })
                        .insert(RigidBody::Fixed)
                        .insert(collider)
                        .id(),
                );
            }
            None => {}
        }
    }

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
            let volexs = chunk_map.get_with_neighbor_full_y(key);
            match gen_mesh(volexs.to_owned(), material_config.clone()) {
                Some((render_mesh, collider)) => {
                    let task = pool.spawn(async move { (key, render_mesh, collider) });
                    mesh_task.tasks.push(task);
                }
                None => {}
            }
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
    }
}
