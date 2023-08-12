use bevy::prelude::*;

pub fn c_root(b: &mut NodeBundle) {
    b.style.width = Val::Percent(100.);
    b.style.height = Val::Percent(100.);
    b.style.flex_direction = FlexDirection::Column;
    b.style.justify_content = JustifyContent::FlexEnd;
}

pub fn c_buttom(b: &mut NodeBundle) {
    let s = &mut b.style;
    s.flex_direction = FlexDirection::Column;
    s.align_items = AlignItems::Center;
    s.align_content = AlignContent::Center;
    s.justify_content = JustifyContent::FlexEnd;
    s.padding = UiRect::all(Val::Px(10.));
}

pub fn c_text(_a: &AssetServer, b: &mut TextBundle) {
    b.style.margin = UiRect::all(Val::Px(10.));
}

// 字体配置
pub fn c_pixel(assets: &AssetServer, s: &mut TextStyle) {
    s.font = assets
        .load("font/ark-pixel-12px-monospaced-zh_hk.ttf")
        .into();
    s.font_size = 12.;
    s.color = Color::BLACK.into();
}

pub fn c_grid(b: &mut NodeBundle) {
    b.style.display = Display::Flex;
    b.style.padding = UiRect::all(Val::Px(1.));
    b.style.margin = UiRect::all(Val::Px(1.));
}

pub fn c_inv_slot(assets: &AssetServer, b: &mut ImageBundle) {
    b.style.top = Val::Px(0.);
    b.style.left = Val::Px(0.);
    b.style.width = Val::Px(64.);
    b.style.height = Val::Px(64.);
    b.image = assets.load("ui/item_slot.png").into();
}

pub fn c_bule(node: &mut NodeBundle) {
    node.background_color = Color::BLUE.into();
}

pub fn c_red(node: &mut NodeBundle) {
    node.background_color = Color::RED.into();
}

pub fn c_yellow(node: &mut NodeBundle) {
    node.background_color = Color::YELLOW.into();
}

pub fn c_toolbar_box_normal(assets: &AssetServer, node: &mut ButtonBundle) {
    node.style.width = Val::Px(68.);
    node.style.height = Val::Px(68.);
    node.style.border = UiRect::all(Val::Px(2.));
    node.background_color = Color::rgba(0., 0., 0., 0.).into();
    node.border_color = Color::rgba(0., 0., 0., 0.).into();
    // node.style.overflow = Overflow::visible();
}

pub fn c_overflow(node: &mut NodeBundle) {
    node.style.position_type = PositionType::Absolute;
}

pub fn c_test_staff(assets: &AssetServer, b: &mut ImageBundle) {
    b.z_index = ZIndex::Global(4);
    // b.style.top = Val::Px(64. - 40.);
    // b.style.left = Val::Px(-64. + 40.);
    b.style.top = Val::Px(0. + 32. - 20.);
    b.style.left = Val::Px(-64. + 32. - 20.);
    b.style.width = Val::Px(40.);
    b.style.height = Val::Px(40.);
    b.background_color = Color::rgba(0., 0., 0., 0.).into();
    // b.image = assets.load("textures/002.png").into();
}
