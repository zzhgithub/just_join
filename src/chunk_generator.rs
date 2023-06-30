use bevy::prelude::{IVec3, Res, ResMut, Resource};
use bevy_egui::egui::Key;
use ndshape::{ConstShape, ConstShape3u32};

use crate::{
    chunk::{find_chunk_keys_by_shpere, generate_offset_array, ChunkKey, NeighbourOffest},
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

    pub fn write_chunk(&mut self, chunk_key: ChunkKey, item: Vec<Voxel>) {
        self.map_data.insert(chunk_key, item);
    }

    pub fn get_by_index(volex: Option<&Vec<Voxel>>, index: u32) -> Voxel {
        match volex {
            Some(list) => list[index as usize],
            None => Voxel::EMPTY,
        }
    }

    // 下面的代码 是有问题的！！！
    // BUG 不能正确的生成数据
    pub fn get_with_neighbor(&mut self, chunk_key: ChunkKey) -> Vec<Voxel> {
        let voxels = self.get(chunk_key);
        // 这个是最核心的 数据
        type SampleShape = ConstShape3u32<18, 18, 18>;
        type DataShape = ConstShape3u32<16, 16, 16>;

        let py = &IVec3::new(0, 1, 0);
        let ny = &IVec3::new(0, -1, 0);
        let px = &IVec3::new(1, 0, 0);
        let nx = &IVec3::new(-1, 0, 0);
        let pz = &IVec3::new(0, 0, 1);
        let nz = &IVec3::new(0, 0, -1);

        let offsets = vec![py, ny, px, nx, pz, nz];
        let mut map: SmallKeyHashMap<IVec3, Vec<Voxel>> = SmallKeyHashMap::new();
        for ele in offsets {
            let new_key = ChunkKey(chunk_key.0 + ele.clone());
            let _ = match self.get(new_key) {
                Some(v) => {
                    map.insert(ele.clone(), v.clone());
                }
                None => (),
            };
        }
        let mut result = Vec::new();
        for i in 0..SampleShape::SIZE {
            let [x, y, z] = SampleShape::delinearize(i);
            if x != 0 && x != 17 && z != 0 && z != 17 && y == 17 {
                // y轴
                let index = DataShape::linearize([x - 1, 0, z - 1]);
                let v = map.get(py);
                result.push(Self::get_by_index(v, index));
            } else if x != 0 && x != 17 && z != 0 && z != 17 && y == 0 {
                let index = DataShape::linearize([x - 1, 16 - 1, z - 1]);
                let v: Option<&Vec<Voxel>> = map.get(ny);
                result.push(Self::get_by_index(v, index));
            } else if y != 0 && y != 17 && z != 0 && z != 17 && x == 17 {
                // y轴
                let index = DataShape::linearize([0, y - 1, z - 1]);
                let v: Option<&Vec<Voxel>> = map.get(px);
                result.push(Self::get_by_index(v, index));
            } else if y != 0 && y != 17 && z != 0 && z != 17 && x == 0 {
                let index = DataShape::linearize([16 - 1, y - 1, z - 1]);
                let v = map.get(nx);
                result.push(Self::get_by_index(v, index));
            } else if x != 0 && x != 17 && y != 0 && y != 17 && z == 17 {
                // z轴
                let index = DataShape::linearize([x - 1, y - 1, 0]);
                let v = map.get(pz);
                result.push(Self::get_by_index(v, index));
            } else if x != 0 && x != 17 && y != 0 && y != 17 && z == 0 {
                let index = DataShape::linearize([x - 1, y - 1, 16 - 1]);
                let v = map.get(nz);
                result.push(Self::get_by_index(v, index));
            } else if x > 0 && x < 17 && y > 0 && y < 17 && z > 0 && z < 17 {
                let index = DataShape::linearize([x - 1, y - 1, z - 1]);
                result.push(Self::get_by_index(voxels, index));
            } else {
                result.push(Voxel::EMPTY);
            }
        }

        result
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
