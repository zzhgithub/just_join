// 使用数据数据

use std::hash::Hash;

use bevy::prelude::{IVec3, Resource};
use ndshape::{ConstShape, ConstShape3u32};
use sled::Db;

use crate::{chunk::ChunkKey, map_generator::gen_chunk_data_by_seed, voxel::Voxel, CHUNK_SIZE_U32};

#[derive(Resource)]
pub struct MapDataBase {
    pub db: Db,
}

impl MapDataBase {
    pub fn new(path: &str) -> Self {
        let db = sled::open(path).unwrap();
        Self { db: db }
    }

    pub fn find_by_chunk_key(&self, chunk_key: ChunkKey) -> Vec<Voxel> {
        let mut voxels = Vec::new();
        type SampleShape = ConstShape3u32<CHUNK_SIZE_U32, CHUNK_SIZE_U32, CHUNK_SIZE_U32>;
        for i in 0..SampleShape::SIZE {
            voxels.push(Voxel::EMPTY);
        }
        let key = chunk_key.as_u8_array();
        return match self.db.get(key) {
            Ok(rs) => match rs {
                Some(data) => bincode::deserialize(&data).unwrap(),
                // 这里在没有获取到的情况下使用算法的值
                None => gen_chunk_data_by_seed(1512354854, chunk_key),
            },
            Err(e) => {
                println!("wrong, to get Map");
                voxels
            }
        };
    }
}
