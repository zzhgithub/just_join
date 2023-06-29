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
    SmallKeyHashMap, CHUNK_SIZE, VIEW_RADIUS,
};

#[derive(Debug, Clone, Resource, Default)]
pub struct MeshManager {
    pub entities: SmallKeyHashMap<ChunkKey, Entity>,
    // pub fast_key: HashSet<ChunkKey>,
}

pub fn update_mesh_system(
    mut commands: Commands,
    chunk_map: Res<ChunkMap>,
    mut mesh_manager: ResMut<MeshManager>,
    clip_spheres: Res<ClipSpheres>,
    neighbour_offest: Res<NeighbourOffest>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    for key in find_chunk_keys_array_by_shpere(clip_spheres.new_sphere, neighbour_offest.0.clone())
        .drain(..)
    {
        if !mesh_manager.entities.contains_key(&key) {
            let volexs = if let Some(v) = chunk_map.get(key) {
                v
            } else {
                return;
            };
            let render_mesh = gen_mesh(volexs.to_owned()).unwrap();
            // mesh_manager.fast_key.remove(&key);
            mesh_manager.entities.insert(
                key,
                commands
                    .spawn(PbrBundle {
                        transform: Transform::from_xyz(
                            (key.0.x * CHUNK_SIZE) as f32,
                            (key.0.y * CHUNK_SIZE) as f32,
                            (key.0.z * CHUNK_SIZE) as f32,
                        ),
                        mesh: mesh_assets.add(render_mesh),
                        material: material_assets
                            .add(StandardMaterial::from(Color::rgb(1.0, 0.0, 0.0))),
                        ..Default::default()
                    })
                    .id(),
            );
        }
    }
    // for chunk_key in mesh_manager.fast_key.drain() {
    //     if let Some(entity) = mesh_manager.entities.remove(&chunk_key) {
    //         commands.entity(entity).despawn();
    //     }
    // }
    // for &chunk_key in mesh_manager.entities.keys() {
    //     mesh_manager.fast_key.insert(chunk_key.clone());
    // }
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
            commands.entity(entity).despawn();
        }
    }
}
