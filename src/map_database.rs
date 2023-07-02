// 使用数据数据

use std::hash::Hash;

use bevy::prelude::{IVec3, Resource};
use ndshape::{ConstShape, ConstShape3u32};
use sled::Db;

use crate::{chunk::ChunkKey, map_generator::gen_chunk_data_by_seed, voxel::Voxel};

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
        type SampleShape = ConstShape3u32<16, 16, 16>;
        for i in 0..SampleShape::SIZE {
            voxels.push(Voxel::EMPTY);
        }
        let key = chunk_key.as_u8_array();
        return match self.db.get(key) {
            Ok(rs) => match rs {
                Some(data) => bincode::deserialize(&data).unwrap(),
                // 这里在没有获取到的情况下使用算法的值
                None => gen_chunk_data_by_seed(15665482, chunk_key),
            },
            Err(e) => {
                println!("wrong, to get Map");
                voxels
            }
        };
    }

    // 测试生成地图的代码
    pub fn test_gen(&mut self) {
        type SampleShape = ConstShape3u32<16, 16, 16>;
        let mut voxels = Vec::new();
        for i in 0..SampleShape::SIZE {
            // let [x, y, z] = SampleShape::delinearize(i);
            // if ((x * x + y * y + z * z) as f32).sqrt() < 16.0 {
            //     voxels.push(Voxel::FILLED);
            // } else {
            //     voxels.push(Voxel::EMPTY);
            // };
            voxels.push(Voxel::FILLED);
        }

        let serialized = bincode::serialize(&voxels).unwrap();
        self.db
            .insert(ChunkKey(IVec3::ZERO).as_u8_array(), serialized.clone());
        self.db.insert(
            ChunkKey(IVec3::new(0, 0, 1)).as_u8_array(),
            serialized.clone(),
        );
    }
}
