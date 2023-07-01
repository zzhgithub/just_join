use ndshape::{ConstShape, ConstShape3u32};

use crate::{chunk::ChunkKey, voxel::Voxel};

pub fn gen_chunk_data_by_seed(seed: i32, chunk_key: ChunkKey) -> Vec<Voxel> {
    // 怎么计算出
    // 这里浅浅的 试一下这个算法
    let base_x = (chunk_key.0.x * 16) as f32;
    let base_y = (chunk_key.0.y * 16) as f32;
    let base_z = (chunk_key.0.z * 16) as f32;
    type SampleShape = ConstShape3u32<16, 16, 16>;
    let mut voxels = Vec::new();
    for i in 0..SampleShape::SIZE {
        let [x, y, z] = SampleShape::delinearize(i);
        let p_x = base_x + x as f32 / 16.0;
        let p_z = base_z + z as f32 / 16.0;
        //
        let p_y = base_y + y as f32 / 16.0;

        // let h = (p_x * 8.0).sin() + (p_z * 8.0).cos();
        if p_y <= 0.0 {
            voxels.push(Voxel::FILLED);
        } else {
            voxels.push(Voxel::EMPTY);
        }
    }
    voxels
}
