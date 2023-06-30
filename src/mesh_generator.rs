use std::collections::HashSet;

use bevy::{
    prelude::{
        Assets, Color, Commands, Entity, Mesh, PbrBundle, Res, ResMut, Resource, StandardMaterial,
        Transform,
    },
    tasks::{AsyncComputeTaskPool, Task},
};

use crate::{
    chunk::{find_chunk_keys_array_by_shpere, ChunkKey, NeighbourOffest},
    chunk_generator::ChunkMap,
    clip_spheres::ClipSpheres,
    mesh::gen_mesh,
    voxel::Voxel,
    SmallKeyHashMap, CHUNK_SIZE, VIEW_RADIUS,
};

#[derive(Debug, Clone, Resource, Default)]
pub struct MeshManager {
    pub entities: SmallKeyHashMap<ChunkKey, Entity>,
    pub fast_key: HashSet<ChunkKey>,
}

#[derive(Debug, Resource)]
pub struct MeshTasks {
    pub tasks: Vec<Task<(ChunkKey, Mesh)>>,
}

pub fn update_mesh_system(
    mut commands: Commands,
    mut chunk_map: ResMut<ChunkMap>,
    mut mesh_manager: ResMut<MeshManager>,
    clip_spheres: Res<ClipSpheres>,
    neighbour_offest: Res<NeighbourOffest>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    mut mesh_task: ResMut<MeshTasks>,
) {
    let pool = AsyncComputeTaskPool::get();
    for ele in mesh_task.tasks.drain(..) {
        let (chunk_key, mesh) = futures_lite::future::block_on(ele);
        mesh_manager.entities.insert(
            chunk_key,
            commands
                .spawn(PbrBundle {
                    transform: Transform::from_xyz(
                        (chunk_key.0.x * CHUNK_SIZE) as f32,
                        (chunk_key.0.y * CHUNK_SIZE) as f32,
                        (chunk_key.0.z * CHUNK_SIZE) as f32,
                    ),
                    mesh: mesh_assets.add(mesh),
                    material: material_assets
                        .add(StandardMaterial::from(Color::rgb(1.0, 0.0, 0.0))),
                    ..Default::default()
                })
                .id(),
        );
    }

    for key in find_chunk_keys_array_by_shpere(clip_spheres.new_sphere, neighbour_offest.0.clone())
        .drain(..)
    {
        if !mesh_manager.entities.contains_key(&key) && !mesh_manager.fast_key.contains(&key) {
            // fixme: 这里需要一个很好的加载周围的算法！
            let volexs = if let Some(v) = chunk_map.get(key) {
                v
            } else {
                return;
            };
            // 无论如何都插入进去 放置下次重复检查
            mesh_manager.fast_key.insert(key);
            match gen_mesh(volexs.to_owned()) {
                Some(render_mesh) => {
                    let task = pool.spawn(async move { (key, render_mesh) });
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
    for key in find_chunk_keys_array_by_shpere(clip_spheres.old_sphere, neighbour_offest.0.clone())
        .drain(..)
    {
        chunks_to_remove.insert(key);
    }

    for key in find_chunk_keys_array_by_shpere(clip_spheres.new_sphere, neighbour_offest.0.clone())
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
