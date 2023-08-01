use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use bevy::prelude::{IVec3, Resource};

use crate::{clip_spheres::Sphere3, CHUNK_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkKey(pub IVec3);

impl ChunkKey {
    pub fn as_u8_array(&self) -> [u8; 8] {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        let hash_value = hasher.finish();
        let hash_bytes: [u8; 8] = unsafe { std::mem::transmute(hash_value) };
        return hash_bytes;
    }
}


// 生成 y 偏移两为0的移动
pub fn generate_offset_array_with_y_0(chunk_distance: i32) -> Vec<IVec3> {
    let mut offsets = Vec::new();
    for x in -chunk_distance as i32..=chunk_distance as i32 {
        for z in -chunk_distance as i32..=chunk_distance as i32 {
            offsets.push(IVec3::new(x, 0, z));
        }
    }

    offsets
}

#[derive(Debug, Resource, Clone)]
pub struct NeighbourOffest(pub Vec<IVec3>);

pub fn generate_offset_resoure(radius: f32) -> NeighbourOffest {
    let chunk_distance = radius as i32 / CHUNK_SIZE;
    let mut offsets = generate_offset_array_with_y_0(chunk_distance);
    // itself
    offsets.push(IVec3::ZERO);

    NeighbourOffest(offsets)
}


pub fn find_chunk_keys_array_by_shpere_y_0(sphere: Sphere3, offsets: Vec<IVec3>) -> Vec<ChunkKey> {
    let mut center_chunk_point = sphere.center.as_ivec3() / CHUNK_SIZE;
    center_chunk_point.y = 0;
    offsets
        .iter()
        .map(|&ele| ChunkKey(center_chunk_point + ele))
        .collect()
}

// offsets 已经改变成了平面为零的情况 数据需要扩展的是y轴
pub fn find_chunk_keys_by_shpere_to_full_height(
    sphere: Sphere3,
    offsets: Vec<IVec3>,
    mut rt: impl FnMut(ChunkKey),
) {
    let mut center_chunk_point = sphere.center.as_ivec3() / CHUNK_SIZE;
    center_chunk_point.y = 0;
    for &ele in offsets.iter() {
        for y_offset in -7..=8 {
            rt(ChunkKey(
                center_chunk_point
                    + ele
                    + IVec3 {
                        x: 0,
                        y: y_offset,
                        z: 0,
                    },
            ))
        }
    }
}
