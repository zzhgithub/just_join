use bevy::{
    prelude::{Mesh, Vec3},
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use bevy_rapier3d::prelude::Collider;
use block_mesh::{greedy_quads, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG};
use ndshape::{ConstShape, ConstShape3u32};

use crate::{
    mesh_material::ATTRIBUTE_DATA, voxel::Voxel, voxel_config::MaterailConfiguration, CHUNK_SIZE,
};

pub fn gen_mesh(
    voxels: Vec<Voxel>,
    material_config: MaterailConfiguration,
) -> Option<(Mesh, Collider)> {
    type SampleShape = ConstShape3u32<18, 256, 18>;
    let mut buffer = GreedyQuadsBuffer::new(SampleShape::SIZE as usize);
    let faces: [block_mesh::OrientedBlockFace; 6] = RIGHT_HANDED_Y_UP_CONFIG.faces;
    // let padding_voxels = padding_extents(voxels);
    greedy_quads(
        &voxels,
        &SampleShape {},
        [0; 3],
        [(CHUNK_SIZE + 1) as u32, 255, (CHUNK_SIZE + 1) as u32],
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
    let mut data = Vec::with_capacity(num_vertices);

    for (block_face_normal_index, (group, face)) in buffer
        .quads
        .groups
        .as_ref()
        .into_iter()
        .zip(faces.into_iter())
        .enumerate()
    {
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
            let index = SampleShape::linearize(quad.minimum);

            // 法向量值
            let normol_num = (block_face_normal_index as u32) << 8u32;
            // 贴图索引
            let txt_index = MaterailConfiguration::find_volex_index(
                material_config.clone(),
                block_face_normal_index as u8,
                &voxels[index as usize].id,
            );
            // voxels[index as usize].id
            // todo 这里后面要知道是那个面的方便渲染
            data.extend_from_slice(&[normol_num | (txt_index) as u32; 4]);
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);

    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    render_mesh.insert_attribute(ATTRIBUTE_DATA, VertexAttributeValues::Uint32(data));
    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    let collider_vertices: Vec<Vec3> = positions.iter().cloned().map(|p| Vec3::from(p)).collect();
    let collider_indices: Vec<[u32; 3]> = indices.chunks(3).map(|i| [i[0], i[1], i[2]]).collect();
    // let collider = ColliderShape::trimesh(collider_vertices, collider_indices);
    let collider = Collider::trimesh(collider_vertices, collider_indices);
    // Collider::trimesh(vertices, indices);
    Some((render_mesh, collider))
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
