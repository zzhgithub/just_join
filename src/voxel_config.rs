// 然后一个地方 可以注册这些方块 然后再一个地方可以编辑 生成 配置文件

// 可以基于这个配置文件去  去读取图片 到一个数据 并且 可以根据 data的值来生成 图片数据的 索引。

// 问题是 有个地方 可以编辑他们  可以直接生成配置文件

// 材质配置对象

use std::io::Write;

use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::InspectorOptions;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    mesh_material::{BindlessMaterial, MaterialStorge},
    voxel::{Grass, Soli, Stone, VoxelMaterial},
    SmallKeyHashMap,
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
                println!("文件路径: {}", file_path.display());
                self.files.push(String::from(
                    file_path.to_str().unwrap().replace("assets/", ""),
                ));
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
                let new_self = res.load_all_voxels();
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
}

// 材质相关的工具
pub struct VoxelMaterialToolPulgin;

impl Plugin for VoxelMaterialToolPulgin {
    fn build(&self, app: &mut App) {
        // TODO 这里要加载 配置项目。 然后还要加载 不同的 UI界面
        app.register_type::<MaterailConfiguration>() // you need to register your type to display it
            .add_plugins(ResourceInspectorPlugin::<MaterailConfiguration>::default())
            .add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<BindlessMaterial>>,
) {
    let config = MaterailConfiguration::new()
        .read_file(String::from("volex.ron"))
        .unwrap();

    commands.insert_resource(MaterialStorge::init_with_files(
        asset_server,
        materials,
        config.files.clone(),
    ));
    commands.insert_resource(config.clone());

    config.write_file(String::from("volex.ron"));
}
