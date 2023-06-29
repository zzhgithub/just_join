use bevy::prelude::{Res, ResMut, Resource};
use ndshape::{ConstShape, ConstShape3u32};

use crate::{
    chunk::{find_chunk_keys_by_shpere, ChunkKey, NeighbourOffest},
    clip_spheres::{self, ClipSpheres},
    voxel::Voxel,
    SmallKeyHashMap,
};

#[derive(Debug, Clone, Default, Resource)]
pub struct ChunkMap {
    pub map_data: SmallKeyHashMap<ChunkKey, Vec<Voxel>>,
}

impl ChunkMap {
    pub fn new() -> Self {
        let data_map = SmallKeyHashMap::<ChunkKey, Vec<Voxel>>::new();
        Self { map_data: data_map }
    }

    pub fn gen_chunk_data(&mut self, chunk_key: ChunkKey) {
        if !self.map_data.contains_key(&chunk_key) {
            type ChunkShape = ConstShape3u32<16, 16, 16>;
            let mut data = Vec::<Voxel>::new();
            // 这里使用最简单的逻辑
            for i in 0..ChunkShape::SIZE {
                let [x, y, z] = ChunkShape::delinearize(i);
                let heigh = chunk_key.0.y * 16 + y as i32;
                if (heigh < -2) {
                    data.push(Voxel::FILLED);
                } else {
                    data.push(Voxel::EMPTY);
                }
            }
            self.write_chunk(chunk_key, data);
        }
    }

    pub fn get(&self, key: ChunkKey) -> Option<&Vec<Voxel>> {
        self.map_data.get(&key)
    }

    pub fn get_with_neighbour(&self, key: ChunkKey) -> Option<&Vec<Voxel>> {
        // 获取 全部数据
        // 然后生成中心点的四周的数据 这个怎么去取形状最后转换成 数组呢？
        None
    }

    pub fn write_chunk(&mut self, chunk_key: ChunkKey, item: Vec<Voxel>) {
        self.map_data.insert(chunk_key, item);
    }
}

pub fn chunk_generate_system(
    mut chunk_map: ResMut<ChunkMap>,
    neighbour_offest: Res<NeighbourOffest>,
    clip_spheres: Res<ClipSpheres>,
) {
    find_chunk_keys_by_shpere(
        clip_spheres.new_sphere,
        neighbour_offest.0.clone(),
        move |key| {
            // 这里要判断一下获取的方法
            chunk_map.gen_chunk_data(key);
        },
    );
}
