use bevy::{
    prelude::{ResMut, StandardMaterial, With, World},
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContext, EguiContexts};
use bevy_inspector_egui::prelude::*;

pub fn inspector_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world) else { return };
    let mut egui_context = egui_context.clone();
    egui::Window::new("UI").show(&egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // equivalent to `WorldInspectorPlugin`
            // bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

            egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(world, ui);
            });

            ui.heading("Entities");
            bevy_inspector_egui::bevy_inspector::ui_for_world_entities(world, ui);
        });
    });
}
