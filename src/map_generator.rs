use std::f32::consts::{E, PI};

use ndshape::{ConstShape, ConstShape2u32, ConstShape3u32};
use simdnoise::NoiseBuilder;

use crate::{chunk::ChunkKey, voxel::Voxel};

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
        let p_x = base_x + x as f32;
        let p_y = base_y + y as f32;
        let p_z = base_z + z as f32;

        // let h = ((p_x * PI / 2.0).sin() + (p_z * PI / 2.0).cos());
        let h = 20.0 * (p_x / 200.0).sin() + 10.0 * (p_z / 100.0).cos() + E.powf(p_x / 3000.0)
            - E.powf(p_z / 3000.0);
        // println!("({},{})", h, p_y);
        let index = PanleShap::linearize([x, z]);
        if p_y <= h + noise[index as usize] * 10.0 {
            if p_y < 5.0 {
                voxels.push(Voxel::soil);
            } else if p_y < 7.0 {
                voxels.push(Voxel::grass);
            } else {
                voxels.push(Voxel::stone);
            }
        } else {
            voxels.push(Voxel::EMPTY);
        }

        // let h = (p_x * 8.0).sin() + (p_z * 8.0).cos();
        // if p_y <= noise[index as usize] * 10.0 {
        //     voxels.push(Voxel::FILLED);
        // } else {
        //     voxels.push(Voxel::EMPTY);
        // }
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
