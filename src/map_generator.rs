use ndshape::{ConstShape, ConstShape2u32, ConstShape3u32};
use simdnoise::NoiseBuilder;

use crate::{chunk::ChunkKey, voxel::Voxel};

// 𝐻𝑒𝑖𝑔ℎ𝑡(𝑥,𝑦)=128∗𝑁𝑜𝑖𝑠𝑒2𝐷(4𝑥,4𝑦)+64∗𝑁𝑜𝑖𝑠𝑒2𝐷(8𝑥,8𝑦)+32∗𝑁𝑜𝑖𝑠𝑒2𝐷(16𝑥,16𝑦)
pub fn gen_chunk_data_by_seed(seed: i32, chunk_key: ChunkKey) -> Vec<Voxel> {
    // 怎么计算出
    // 这里浅浅的 试一下这个算法
    let base_x = (chunk_key.0.x * 16) as f32;
    let base_y = (chunk_key.0.y * 16) as f32;
    let base_z = (chunk_key.0.z * 16) as f32;
    type SampleShape = ConstShape3u32<16, 16, 16>;
    type PanleShap = ConstShape2u32<16, 16>;
    let mut voxels = Vec::new();

    let noise = noise2d(chunk_key, seed);

    for i in 0..SampleShape::SIZE {
        let [x, y, z] = SampleShape::delinearize(i);
        let p_y = base_y + y as f32 / 16.0;
        let index = PanleShap::linearize([x, z]);
        // let h = (p_x * 8.0).sin() + (p_z * 8.0).cos();
        if p_y <= noise[index as usize] * 5.0 {
            voxels.push(Voxel::FILLED);
        } else {
            voxels.push(Voxel::EMPTY);
        }
    }
    voxels
}

// 生成2d的柏林噪声
pub fn noise2d(chunk_key: ChunkKey, seed: i32) -> Vec<f32> {
    let (noise, _max, _min) = NoiseBuilder::fbm_2d_offset(
        (chunk_key.0.x * 16) as f32,
        16,
        (chunk_key.0.z * 16) as f32,
        16,
    )
    .with_seed(seed)
    .with_freq(0.25)
    .with_octaves(6)
    .generate();
    noise
}
