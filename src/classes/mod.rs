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
    node.style.border = UiRect::all(Val::Px(2.));
    node.background_color = Color::rgba(0., 0., 0., 0.).into();
    node.border_color = Color::rgba(0., 0., 0., 0.).into();
}
