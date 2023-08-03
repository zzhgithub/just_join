use std::f32::consts::{E, PI};

use ndshape::{ConstShape, ConstShape2u32, ConstShape3u32};
use simdnoise::NoiseBuilder;

use crate::{
    chunk::ChunkKey,
    voxel::{Grass, Soli, Sown, Stone, Voxel, VoxelMaterial, Water},
};

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
    let noise2 = noise2d_ridge(chunk_key, seed);

    for i in 0..SampleShape::SIZE {
        let [x, y, z] = SampleShape::delinearize(i);
        let p_x = base_x + x as f32;
        let p_y = base_y + y as f32;
        let p_z = base_z + z as f32;

        // let h = ((p_x * PI / 2.0).sin() + (p_z * PI / 2.0).cos());
        let h = 20.0 * (p_x / 200.0).sin() + 10.0 * (p_z / 100.0).cos();
        //  + E.powf(p_x / 3000.0)- E.powf(p_z / 3000.0);
        // println!("({},{})", h, p_y);
        let index = PanleShap::linearize([x, z]);
        let top = h + noise[index as usize] * 10.0 + noise2[index as usize] * 5.0;
        if p_y <= top {
            if p_y >= 40. {
                voxels.push(Sown::into_voxel());
                continue;
            }
            if p_y >= 35. {
                voxels.push(Stone::into_voxel());
                continue;
            }
            if p_y >= top - 1.0 {
                voxels.push(Grass::into_voxel());
            } else if p_y > top - 5.0 {
                voxels.push(Soli::into_voxel());
            } else {
                voxels.push(Stone::into_voxel());
            }
        } else {
            voxels.push(Voxel::EMPTY);
        }
    }
    //  侵蚀 洞穴
    let noise_3d = noise3d_2(chunk_key, seed);
    for i in 0..SampleShape::SIZE {
        // let [x, y, z] = SampleShape::delinearize(i);
        // let index = SampleShape::linearize([x, z, y]);
        let flag: f32 = noise_3d[i as usize];
        if flag < 0.05 && flag > -0.05 {
            voxels[i as usize] = Voxel::EMPTY;
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

pub fn noise3d(chunk_key: ChunkKey, seed: i32) -> Vec<f32> {
    let (noise, _max, _min) = NoiseBuilder::fbm_3d_offset(
        (chunk_key.0.x * 16) as f32,
        16,
        (chunk_key.0.y * 16) as f32,
        16,
        (chunk_key.0.z * 16) as f32,
        16,
    )
    .with_seed(seed)
    .with_freq(0.1)
    .with_octaves(5)
    // .with_gain(2.0)
    .with_lacunarity(0.5)
    .generate();
    noise
}

pub fn noise2d_ridge(chunk_key: ChunkKey, seed: i32) -> Vec<f32> {
    let (noise, min, max) = NoiseBuilder::ridge_2d_offset(
        (chunk_key.0.x * 16) as f32,
        16,
        (chunk_key.0.z * 16) as f32,
        16,
    )
    .with_seed(seed)
    .with_freq(0.05)
    .with_octaves(5)
    .with_gain(2.0)
    .with_lacunarity(0.5)
    .generate();
    noise
}

// todo: 尝试产生 洞穴的噪声
pub fn noise3d_2(chunk_key: ChunkKey, seed: i32) -> Vec<f32> {
    let (noise, min, max) = NoiseBuilder::fbm_3d_offset(
        (chunk_key.0.x * 16) as f32,
        16,
        (chunk_key.0.y * 16) as f32,
        16,
        (chunk_key.0.z * 16) as f32,
        16,
    )
    .with_seed(seed)
    .with_freq(0.2)
    .with_lacunarity(0.5)
    .with_gain(2.0)
    .with_octaves(6)
    .generate();
    noise
}
