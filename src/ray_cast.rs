use bevy::{
    pbr::wireframe::Wireframe,
    prelude::{
        shape::Cube, AlphaMode, Assets, Color, Commands, Component, GlobalTransform, Mesh,
        PbrBundle, Plugin, Query, Res, ResMut, Resource, StandardMaterial, Startup, Transform,
        Update, Vec3, Visibility, With, Without,
    },
    render::render_resource::PrimitiveTopology,
};
use bevy_egui::egui::color_picker::Alpha;
use bevy_rapier3d::prelude::{QueryFilter, RapierContext};
use controller::controller::CameraTag;

use crate::{
    palyer::{PlayerController, PlayerStorge},
    player_controller::PlayerMe,
};

fn get_pos_chunk_center(vec3: Vec3, normal: Vec3) -> Vec3 {
    // 应该是命中点所在的面的中点
    let mid_pos = Vec3::new(
        vec3.x.floor() + 0.5 * (if normal.x == 0.0 { 1.0 } else { 0.0 }),
        vec3.y.floor() + 0.5 * (if normal.y == 0.0 { 1.0 } else { 0.0 }),
        vec3.z.floor() + 0.5 * (if normal.z == 0.0 { 1.0 } else { 0.0 }),
    );
    mid_pos - (normal * 0.5)
}

pub fn touth_mesh_ray_cast(
    query: Query<&GlobalTransform, With<CameraTag>>,
    rapier_context: Res<RapierContext>,
    player_storge: Res<PlayerStorge>,
    mut choose_cube: ResMut<ChooseCube>,
    mut query_help_cube: Query<
        (&mut Transform, &mut Visibility),
        (With<HelpCube>, Without<CameraTag>),
    >,
    // mut query_visibility: Query<&mut Visibility, With<HelpCube>>,
) {
    let Ok((mut chue_pos,mut visibility)) = query_help_cube.get_single_mut() else{
        println!("not found CameraTag.");
        return;
    };
    // let Ok(mut visibility) = query_visibility.get_single() else{return;};

    //  这里需要知道当前相机的位置
    let Ok(tfr) = query.get_single() else {
        println!("not found CameraTag");
        return;};
    let ray_pos = tfr.translation();
    let ray_dir = tfr.forward();
    // println!("ray_pos: {:?}", ray_pos);
    // println!("ray_dir {:?}", ray_dir);
    let max_toi = 3.0;
    let solid = true;
    // 这个参数是做什么的？
    let filter: QueryFilter<'_> = QueryFilter::exclude_dynamic()
        .exclude_sensors()
        .exclude_rigid_body(player_storge.0)
        .exclude_collider(player_storge.0);

    let hit = rapier_context.cast_ray_and_get_normal(ray_pos, ray_dir, max_toi, solid, filter);
    match hit {
        Some((entity, intersect)) => {
            // println!("物体检查了");
            // 这里物体没有移动的情况下 可以不处理
            // 选择了物体
            let hit_point = intersect.point;
            let normal = intersect.normal;
            let center_point = get_pos_chunk_center(hit_point, normal);
            let out_center_point = get_pos_chunk_center(hit_point, -normal);

            if let Some(old_center) = choose_cube.center {
                if old_center.distance(center_point) <= 0.0 {
                    // println!("物体没有被移动");
                    return;
                }
            }
            // 设置可见
            *visibility = Visibility::Visible;
            // 设置位置

            *chue_pos = Transform::from_translation(center_point);
            // 设置选中点
            choose_cube.choose_on = Some(hit_point);
            choose_cube.center = Some(center_point);
            choose_cube.out_center = Some(out_center_point);
            // println!("normal {:?}", normal);
        }
        None => {
            // println!("移动出了焦点");
            // 设置不可见
            *visibility = Visibility::Hidden;
            // 设置没有选中点
            choose_cube.choose_on = None;
            choose_cube.center = None;
            choose_cube.out_center = None;
        }
    }
}

#[derive(Component)]
pub struct HelpCube;

#[derive(Resource, Debug, Clone, Copy)]
pub struct ChooseCube {
    // 选中的点
    pub choose_on: Option<Vec3>,
    // 选择中的点对应的方块
    pub center: Option<Vec3>,
    // 选中点 法向量对面的方块
    pub out_center: Option<Vec3>,
}

impl ChooseCube {
    fn new() -> Self {
        Self {
            choose_on: None,
            center: None,
            out_center: None,
        }
    }
}
// 初始化cube

pub fn setup_cube(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: mesh_assets.add(create_cube_wireframe(1.01)),
            visibility: bevy::prelude::Visibility::Hidden,
            material: materials.add(StandardMaterial {
                unlit: true,
                base_color: Color::BLACK,
                depth_bias: 9999.0,
                alpha_mode: AlphaMode::Mask(0.5),
                ..Default::default()
            }), // 使用 Wireframe 材质
            transform: Transform::from_translation(Vec3::ZERO),
            ..Default::default()
        })
        .insert(HelpCube);
}
pub struct MyRayCastPlugin;

impl Plugin for MyRayCastPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // 加载资源
        app.insert_resource(ChooseCube::new())
            .add_systems(Startup, setup_cube)
            .add_systems(Update, touth_mesh_ray_cast);
        // 设置更新系统
    }
}

fn create_cube_wireframe(size: f32) -> Mesh {
    let half_size = size / 2.0;

    let vertices = vec![
        // Front face
        [half_size, half_size, half_size],
        [-half_size, half_size, half_size],
        [-half_size, -half_size, half_size],
        [half_size, -half_size, half_size],
        [half_size, half_size, half_size], // Closing line to complete the wireframe
        // Back face
        [half_size, half_size, -half_size],
        [-half_size, half_size, -half_size],
        [-half_size, -half_size, -half_size],
        [half_size, -half_size, -half_size],
        [half_size, half_size, -half_size], // Closing line to complete the wireframe
        // Connecting lines between front and back faces
        [-half_size, half_size, half_size],
        [-half_size, half_size, -half_size],
        [-half_size, -half_size, half_size],
        [-half_size, -half_size, -half_size],
        [half_size, -half_size, half_size],
        [half_size, -half_size, -half_size],
        [half_size, half_size, half_size],
        [half_size, half_size, -half_size],
    ];

    let indices = vec![
        0, 1, 1, 2, 2, 3, 3, 4, 4, 0, // Front face
        5, 6, 6, 7, 7, 8, 8, 9, 9, 5, // Back face
        10, 11, 12, 13, 14, 15, // Connecting lines
        16, 17,
    ];

    let mut mesh = Mesh::new(PrimitiveTopology::LineList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
    return mesh.into();
}
