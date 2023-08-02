// 然后一个地方 可以注册这些方块 然后再一个地方可以编辑 生成 配置文件

// 可以基于这个配置文件去  去读取图片 到一个数据 并且 可以根据 data的值来生成 图片数据的 索引。

// 问题是 有个地方 可以编辑他们  可以直接生成配置文件

// 材质配置对象

use std::io::Write;

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
    utils::HashMap,
};
use bevy_inspector_egui::InspectorOptions;
use block_mesh::{greedy_quads, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG};
use ndshape::{ConstShape, ConstShape3u32};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    mesh_material::{BindlessMaterial, MaterialStorge, ATTRIBUTE_DATA},
    palyer::PlayerPlugin,
    voxel::{Grass, Soli, Stone, Voxel, VoxelMaterial},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct VoxelConfig {
    pub index: u32,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, InspectorOptions, Reflect)]
#[reflect(InspectorOptions)]
pub struct VoxelTypeConfig {
    pub type_name: String,
    pub type_ch_name: String,
    // 默认配置
    pub default: VoxelConfig,
    // 各个法向量下的配置
    pub normal: HashMap<u8, VoxelConfig>,
}

use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

#[derive(Debug, Clone, Serialize, Deserialize, Resource, InspectorOptions, Default, Reflect)]
#[reflect(Resource, InspectorOptions)]

pub struct MaterailConfiguration {
    // 体素类型列表
    pub voxels: HashMap<u8, VoxelTypeConfig>,
    // 文件地址列表
    pub files: Vec<String>,
}

#[macro_export]
macro_rules! add_volex {
    ($types: ident,$class: expr) => {
        if (!$class.voxels.contains_key(&$types::ID)) {
            $class.voxels.insert(
                $types::ID,
                VoxelTypeConfig {
                    type_name: String::from($types::NAME),
                    type_ch_name: String::from($types::CN_NAME),
                    ..Default::default()
                },
            );
            println!(
                "* 加载体素[{}][{}]",
                String::from($types::NAME),
                String::from($types::CN_NAME),
            );
        } else {
            println!(
                "加载体素[{}][{}]",
                String::from($types::NAME),
                String::from($types::CN_NAME),
            );
        }
    };
}

impl MaterailConfiguration {
    // 初始化
    pub fn new() -> Self {
        // 读取文件夹下的
        Self {
            voxels: HashMap::default(),
            files: Vec::new(),
        }
    }

    // 加载文件夹下的数据
    pub fn load_pic_files(mut self, dir: String) -> Self {
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let file_path = entry.path();
                // 在这里处理文件，例如打印文件路径

                let path = String::from(file_path.to_str().unwrap().replace("assets/", ""));
                if (self.files.contains(&path)) {
                    self.files.push(path);
                    println!("* 文件路径: {}", file_path.display());
                } else {
                    println!("文件路径: {}", file_path.display());
                }
            }
        }
        self
    }

    pub fn load_all_voxels(mut self) -> Self {
        // 初始化全部的数据
        add_volex!(Stone, self);
        add_volex!(Soli, self);
        add_volex!(Grass, self);
        // todo 加载其他的类型
        self
    }

    pub fn read_file(self, path: String) -> Result<Self, ron::Error> {
        let reader = std::fs::File::open(path);
        match reader {
            Ok(file) => {
                // 如果成功取配置
                let res: Self = ron::de::from_reader(file).unwrap();
                let mut new_self = res.load_all_voxels();
                // new_self = self.load_pic_files(String::from("assets/textures"));
                Ok(new_self)
            }
            Err(_) => {
                print!("没有找配置文件第一次加载");
                let mut new_self = self.load_pic_files(String::from("assets/textures"));
                new_self = new_self.load_all_voxels();
                Ok(new_self)
            }
        }
    }

    pub fn write_file(self, path: String) {
        let res = ron::to_string(&self).unwrap();
        let mut file = std::fs::File::create(path).expect("create failed");
        file.write_all(res.as_bytes()).unwrap();
    }

    // 通过面 和 体素类型获取 图片的索引
    pub fn find_volex_index(self, normal: u8, volex_type: &u8) -> u32 {
        return match self.voxels.get(volex_type) {
            Some(config) => {
                return match config.normal.get(&normal) {
                    Some(vconfig) => vconfig.index,
                    None => config.default.index,
                };
            }
            None => 0,
        };
    }
}

// 材质相关的工具
pub struct VoxelMaterialToolPulgin;

impl Plugin for VoxelMaterialToolPulgin {
    fn build(&self, app: &mut App) {
        // TODO 这里要加载 配置项目。 然后还要加载 不同的 UI界面
        app.register_type::<MaterailConfiguration>() // you need to register your type to display it
            .add_plugins(ResourceInspectorPlugin::<MaterailConfiguration>::default())
            // 老版和 物理无关的移动
            .add_plugins(PlayerPlugin)
            .add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<BindlessMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let config = MaterailConfiguration::new()
        .read_file(String::from("volex.ron"))
        .unwrap();

    let storge = MaterialStorge::init_with_files(asset_server, materials, config.files.clone());
    let mat = storge.0.clone();
    commands.insert_resource(storge);
    commands.insert_resource(config.clone());

    // FIXME: 这里后续其他地方调用
    // config.clone().write_file(String::from("volex.ron"));

    type SampleShape = ConstShape3u32<22, 22, 22>;

    let mut voxels = [Voxel::EMPTY; SampleShape::SIZE as usize];
    // 这里填充数据？
    for z in 1..21 {
        for y in 1..21 {
            for x in 1..21 {
                let i = SampleShape::linearize([x, y, z]);
                if ((x * x + y * y + z * z) as f32).sqrt() < 20.0 {
                    if y < 5 {
                        voxels[i as usize] = Grass::into_voxel();
                    } else if y < 10 {
                        voxels[i as usize] = Stone::into_voxel();
                    } else {
                        voxels[i as usize] = Soli::into_voxel();
                    }
                }
            }
        }
    }

    // 21 x 21 x 21

    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    let mut buffer = GreedyQuadsBuffer::new(voxels.len());
    greedy_quads(
        &voxels,
        &SampleShape {},
        [0; 3],
        [21; 3],
        &faces,
        &mut buffer,
    );
    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
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

            // 计算出 data
            // let a: [u32; 3] = quad.minimum.map(|x| x - 1);
            let a = quad.minimum;
            let index = SampleShape::linearize(a);
            // 体素类型值
            let voxel_type = voxels[index as usize].id;
            // 这里的 d 是法向量
            let d = (block_face_normal_index as u32) << 8u32;
            let index = MaterailConfiguration::find_volex_index(
                config.clone(),
                block_face_normal_index as u8,
                &voxel_type,
            );

            // todo 这里后面要知道是那个面的方便渲染
            data.extend_from_slice(&[d | index; 4]);
            // data.extend_from_slice(&[(block_face_normal_index as u32) << 8u32 | c; 4],);
            // &[voxels[index as usize].0 as u32; 4],);
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // 这里没有缩放 每个格子会占据一个图片？

    // for uv in tex_coords.iter_mut() {
    //     for c in uv.iter_mut() {
    //         *c *= UV_SCALE;
    //     }
    // }

    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    render_mesh.insert_attribute(ATTRIBUTE_DATA, VertexAttributeValues::Uint32(data));
    render_mesh.set_indices(Some(Indices::U32(indices)));

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(render_mesh),
        material: mat.clone(),
        transform: Transform::from_translation(Vec3::splat(-10.0)),
        ..Default::default()
    });

    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 50.0)),
        point_light: PointLight {
            range: 200.0,
            intensity: 20000.0,
            ..Default::default()
        },
        ..Default::default()
    });
}
