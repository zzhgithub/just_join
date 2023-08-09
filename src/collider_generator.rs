// 对物理引擎的对象进行生成
// 生成要处理的物理引擎对象
// 删除多余的物理引擎对象 只有角色的一周才生成这个东西！ 或者这里单独的管理 不用那么大的空间!!
use bevy::{
    prelude::{
        Assets, Commands, Component, Entity, GlobalTransform, IntoSystemConfigs, Last, Mesh,
        Plugin, Res, ResMut, Resource, SystemSet, Transform, Update, Vec3,
    },
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_rapier3d::prelude::{Collider, RigidBody};
use std::collections::HashSet;

use crate::{
    chunk::{find_chunk_keys_array_by_shpere_y_0, generate_offset_array_with_y_0, ChunkKey},
    clip_spheres::ClipSpheres,
    mesh_generator::{MeshManager, MeshSystem},
    SmallKeyHashMap, CHUNK_SIZE,
};

#[derive(Debug, Component)]
pub struct TerrainPhysics;

// 管理碰撞体的组件
#[derive(Debug, Resource)]
pub struct ColliderManager {
    pub entities: SmallKeyHashMap<ChunkKey, Entity>,
}

#[derive(Debug, Resource, Default)]
pub struct ColliderTasksManager {
    pub tasks: Vec<Task<(ChunkKey, Collider)>>,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ColliderSystem {
    COLLIDER_TASK,
    COLLIDER_SPAWN,
    COLLIDER_DESPAWN,
}

// 通过当前位置更新要显示的物理结构
pub fn update_collider(
    mut mesh_manager: ResMut<MeshManager>,
    mut collider_manager: ResMut<ColliderManager>,
    mut collider_tasks: ResMut<ColliderTasksManager>,
    meshes: Res<Assets<Mesh>>,
    clip_spheres: Res<ClipSpheres>,
) {
    for chunk_key in find_chunk_keys_array_by_shpere_y_0(
        clip_spheres.new_sphere,
        generate_offset_array_with_y_0(2),
    )
    .drain(..)
    {
        // 这里的每个key就是要生成的
        let pool = AsyncComputeTaskPool::get();

        if (mesh_manager.mesh_storge.contains_key(&chunk_key)
            && !collider_manager.entities.contains_key(&chunk_key))
        {
            let mesh_handle = mesh_manager.mesh_storge.get(&chunk_key).unwrap();
            if let Some(mesh) = meshes.get(mesh_handle) {
                // 生成 collider 碰撞体
                if let Some(positions) = mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                {
                    let indices: Vec<u32> =
                        mesh.indices().unwrap().iter().map(|x| x as u32).collect();
                    let collider_vertices: Vec<Vec3> =
                        positions.iter().cloned().map(|p| Vec3::from(p)).collect();
                    let collider_indices: Vec<[u32; 3]> =
                        indices.chunks(3).map(|i| [i[0], i[1], i[2]]).collect();
                    let collider: Collider = Collider::trimesh(collider_vertices, collider_indices);
                    let task = pool.spawn(async move { (chunk_key, collider) });
                    collider_tasks.tasks.push(task);
                }
            }
        }
    }
}

// 添加碰撞体
pub fn spawn_collider(
    mut collider_tasks: ResMut<ColliderTasksManager>,
    mut collider_manager: ResMut<ColliderManager>,
    mut commands: Commands,
) {
    for ele in collider_tasks.tasks.drain(..) {
        match futures_lite::future::block_on(futures_lite::future::poll_once(ele)) {
            Some((chunk_key, collider)) => {
                let entity = commands
                    .spawn((
                        TerrainPhysics,
                        Transform::from_xyz(
                            (chunk_key.0.x * CHUNK_SIZE) as f32 - CHUNK_SIZE as f32 / 2.0 - 1.0,
                            -128.0 + CHUNK_SIZE as f32 / 2.0,
                            (chunk_key.0.z * CHUNK_SIZE) as f32 - CHUNK_SIZE as f32 / 2.0 - 1.0,
                        ),
                        GlobalTransform::default(),
                    ))
                    .insert(RigidBody::Fixed)
                    .insert(collider)
                    .id();
                collider_manager.entities.insert(chunk_key, entity);
            }
            None => {}
        }
    }
}

pub fn despawn_collider(
    clip_spheres: Res<ClipSpheres>,
    mut collider_manager: ResMut<ColliderManager>,
    mut commands: Commands,
) {
    let neighbour_offest = generate_offset_array_with_y_0(2);
    let mut chunks_to_remove = HashSet::new();
    for key in
        find_chunk_keys_array_by_shpere_y_0(clip_spheres.old_sphere, neighbour_offest.clone())
            .drain(..)
    {
        chunks_to_remove.insert(key);
    }

    for key in
        find_chunk_keys_array_by_shpere_y_0(clip_spheres.new_sphere, neighbour_offest.clone())
            .drain(..)
    {
        chunks_to_remove.remove(&key);
    }

    for chunk_key in chunks_to_remove.into_iter() {
        if let Some(entity) = collider_manager.entities.remove(&chunk_key) {
            commands.entity(entity).despawn();
        }
    }
}

pub struct TerrainPhysicsPlugin;

impl Plugin for TerrainPhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // 这来处理一下物体的 物理引擎实体过多的问题
        app.insert_resource(ColliderManager {
            entities: SmallKeyHashMap::default(),
        })
        .insert_resource(ColliderTasksManager::default())
        .add_systems(
            Update,
            update_collider
                .in_set(ColliderSystem::COLLIDER_TASK)
                .after(MeshSystem::UPDATE_MESH),
        )
        .add_systems(
            Update,
            spawn_collider
                .in_set(ColliderSystem::COLLIDER_SPAWN)
                .after(ColliderSystem::COLLIDER_TASK),
        )
        .add_systems(
            Last,
            despawn_collider.in_set(ColliderSystem::COLLIDER_SPAWN),
        );
    }
}
