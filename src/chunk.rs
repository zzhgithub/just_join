use bevy::prelude::{IVec3, Resource};

use crate::{clip_spheres::Sphere3, CHUNK_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkKey(pub IVec3);

// 这个方法用于生成必要的偏移量
pub fn generate_offset_array(chunk_distance: i32) -> Vec<IVec3> {
    let mut offsets = Vec::new();
    for x in -chunk_distance as i32..=chunk_distance as i32 {
        for y in -chunk_distance as i32..=chunk_distance as i32 {
            for z in -chunk_distance as i32..=chunk_distance as i32 {
                offsets.push(IVec3::new(x, y, z));
            }
        }
    }

    offsets
}

#[derive(Debug, Resource, Clone)]
pub struct NeighbourOffest(pub Vec<IVec3>);

pub fn generate_offset_resoure(radius: f32) -> NeighbourOffest {
    let chunk_distance = radius as i32 / CHUNK_SIZE;
    let mut offsets = generate_offset_array(chunk_distance);
    // itself
    offsets.push(IVec3::ZERO);

    NeighbourOffest(offsets)
}

pub fn find_chunk_keys_by_shpere(
    sphere: Sphere3,
    offsets: Vec<IVec3>,
    mut rt: impl FnMut(ChunkKey),
) {
    let center_chunk_point = sphere.center.as_ivec3() / CHUNK_SIZE;
    for &ele in offsets.iter() {
        rt(ChunkKey(center_chunk_point + ele))
    }
}

pub fn find_chunk_keys_array_by_shpere(sphere: Sphere3, offsets: Vec<IVec3>) -> Vec<ChunkKey> {
    let center_chunk_point = sphere.center.as_ivec3() / CHUNK_SIZE;
    offsets
        .iter()
        .map(|&ele| ChunkKey(center_chunk_point + ele))
        .collect()
}
