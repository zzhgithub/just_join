use bevy::{
    prelude::Mesh,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG};
use ndshape::{ConstShape, ConstShape3u32};

use crate::{voxel::Voxel, CHUNK_SIZE};

pub fn gen_mesh(voxels: Vec<Voxel>) -> Option<Mesh> {
    type SampleShape = ConstShape3u32<18, 18, 18>;
    let mut buffer = GreedyQuadsBuffer::new(SampleShape::SIZE as usize);
    let faces: [block_mesh::OrientedBlockFace; 6] = RIGHT_HANDED_Y_UP_CONFIG.faces;
    // let padding_voxels = padding_extents(voxels);
    greedy_quads(
        &voxels,
        &SampleShape {},
        [0; 3],
        [(CHUNK_SIZE + 1) as u32; 3],
        &faces,
        &mut buffer,
    );
    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
    if num_indices == 0 {
        return None;
    }
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut tex_coords = Vec::with_capacity(num_vertices);
    for (group, face) in buffer.quads.groups.into_iter().zip(faces.into_iter()) {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
            normals.extend_from_slice(&face.quad_mesh_normals());
            tex_coords.extend_from_slice(&face.tex_coords(
                RIGHT_HANDED_Y_UP_CONFIG.u_flip_face,
                true,
                &quad,
            ));
            // 这里可以生成Data???? 但是怎么知道 是那个面的？
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);

    for uv in tex_coords.iter_mut() {
        for c in uv.iter_mut() {
            *c *= CHUNK_SIZE as f32;
        }
    }

    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    render_mesh.set_indices(Some(Indices::U32(indices)));
    Some(render_mesh)
}

pub fn padding_extents(voxels: Vec<Voxel>) -> Vec<Voxel> {
    // 给数据加上Empty的边界
    type SampleShape = ConstShape3u32<18, 18, 18>;
    type DataShape = ConstShape3u32<16, 16, 16>;
    let chunk_size_u32 = CHUNK_SIZE as u32;
    let mut result = Vec::new();
    for i in 0..SampleShape::SIZE {
        let [x, y, z] = SampleShape::delinearize(i);
        if x < 1 || y < 1 || z < 1 || x > chunk_size_u32 || y > chunk_size_u32 || z > chunk_size_u32
        {
            result.push(Voxel::EMPTY);
        } else {
            let index = DataShape::linearize([x - 1, y - 1, z - 1]);
            result.push(voxels[index as usize]);
        }
    }
    return result;
}
