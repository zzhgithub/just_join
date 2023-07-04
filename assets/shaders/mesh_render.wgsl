#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::pbr_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions

#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::pbr_ambient
#import bevy_pbr::shadows
#import bevy_pbr::fog
#import bevy_pbr::pbr_functions


var<private> VOXEL_NORMALS: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
    vec3<f32>(-1., 0., 0.),
    vec3<f32>(0., -1., 0.),
    vec3<f32>(0., 0., -1.), 
    vec3<f32>(1., 0., 0.), 
    vec3<f32>(0., 1., 0.), 
    vec3<f32>(0., 0., 1.), 
);

// Extracts the normal face index from the encoded voxel data
fn voxel_data_extract_normal(voxel_data: u32) -> vec3<f32> {
    return VOXEL_NORMALS[voxel_data >> 8u & 7u];
}

// fn voxel_data_extract_position(voxel_data: u32) -> vec3<f32> {
//     return vec3<f32>(
//         f32(voxel_data >> 27u),
//         f32(voxel_data >> 22u & 31u),
//         f32(voxel_data >> 17u & 31u)
//     );
// }

// Extracts the material index from the encoded voxel data
fn voxel_data_extract_material_index(voxel_data: u32) -> u32 {
    return voxel_data & 255u;
}




// @group(1) @binding(0)
// var my_array_texture: texture_2d_array<f32>;
// @group(1) @binding(1)
// var my_array_texture_sampler: sampler;

@group(1) @binding(0)
var textures: binding_array<texture_2d<f32>>;
@group(1) @binding(1)
var nearest_sampler: sampler;


struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) voxel_data: u32,
    @location(2) uv:vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) voxel_normal: vec3<f32>,
    @location(1) voxel_data: u32,
    @location(2) world_position: vec3<f32>,
    @location(3) uv:vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let world_position = mesh_position_local_to_world(mesh.model, vec4<f32>(vertex.position, 1.0));

    var out: VertexOutput;
    out.clip_position = mesh_position_world_to_clip(world_position);
    out.voxel_normal = voxel_data_extract_normal(vertex.voxel_data);
    out.voxel_data = vertex.voxel_data;
    out.world_position = world_position.xyz;
    out.uv = vertex.uv;
    return out;
}



struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    @location(0) voxel_normal: vec3<f32>,
    /// The voxel data.
    @location(1) voxel_data: u32,
    /// The world position of the voxel vertex.
    @location(2) world_position: vec3<f32>,
    // #import bevy_pbr::mesh_vertex_output
    @location(3) uv:vec2<f32>,
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    // let layer = i32(in.world_position.x) & 0x3;
    let layer = i32(voxel_data_extract_material_index(in.voxel_data));
    let uv = in.uv;
    // let coords = clamp(vec2<u32>(uv * 4.0), vec2<u32>(0u), vec2<u32>(3u));
    // let inner_uv = fract(uv * 4.0);
    

    // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
    // the material members
    var pbr_input: PbrInput = pbr_input_new();
    
    pbr_input.material.base_color = textureSample(textures[layer], nearest_sampler, in.uv);

    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position =  vec4<f32>(in.world_position, 1.0);
    pbr_input.world_normal = (f32(in.is_front) * 2.0 - 1.0) * mesh_normal_local_to_world(in.voxel_normal);

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = normalize(mesh_normal_local_to_world(in.voxel_normal));
    pbr_input.V = calculate_view(vec4<f32>(in.world_position, 1.0), pbr_input.is_orthographic);
    
    return tone_mapping(pbr(pbr_input));
}



