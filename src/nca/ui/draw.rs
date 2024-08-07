//! UI for settings relating to drawing on screen

use bevy::prelude::*;
use bevy_egui::{egui::{self, color_picker::color_edit_button_rgb}, EguiContexts};

use super::super::pipeline::draw::NCADrawSettings;

// =================================== Plugin =================================== //

/// A plugin to manage the UI window for draw settings.
pub(super) struct UIDrawPlugin;

impl Plugin for UIDrawPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, draw_settings_ui);
    }
}

// ================================== Systems =================================== //

/// A system that creates and manages the UI window for draw settings. Lets the user
/// change the brush size, type and color.
fn draw_settings_ui(
    mut contexts: EguiContexts,
    mut draw_params: ResMut<NCADrawSettings>,
) {
    egui::Window::new("Draw Settings").show(contexts.ctx_mut(), |ui| {
        egui::Grid::new("Draw Grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.add(egui::DragValue::new(
                    &mut draw_params.brush_size,
                    ).range(0..=300).clamp_to_range(true)
                );
                ui.label("Brush Size");
                ui.end_row();

                egui::ComboBox::from_id_source("Brush Type")
                .selected_text(match draw_params.brush_type {
                    0 => "Circle",
                    1 => "Square",
                    _ => "",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut draw_params.brush_type, 0, "Circle");
                    ui.selectable_value(&mut draw_params.brush_type, 1, "Square");
                });
                ui.label("Brush Type");
                ui.end_row();

                color_edit_button_rgb(ui, &mut draw_params.brush_color);
                ui.label("Brush Color");
            });
    });
}