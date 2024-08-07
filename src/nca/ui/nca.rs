//! UI for settings relating to the NCA

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use dialog::DialogBox;

use super::super::{
    nca_control::{
        presets::{AddPresetFilter, AddPresetFn, NCAPresets},
        settings::{NCAChannel, NCASettings, SaveSettings},
        Reinitialize,
        UpdateActivationFunction,
        UpdateFilter,
    },
    utils::{array_to_mat3, mat3_to_array}
};

// =================================== Plugin =================================== //

/// A plugin providing a UI window to control the NCA settings.
/// The user can:
///     -change the filter of the NCA in each color channel, which are
///     displayed as drag values in a 3x3-formation (corresponding to the structure
///     of the filter matrix).
///     -change the activation function by writing a function f32 -> f32 in WGSL
///     inside a multiline text edit.
///     -save and load presets for both, filters and activation functions.
pub(super) struct UINCAPlugin;

impl Plugin for UINCAPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<FilterChanged>()
            .add_event::<FunctionChanged>()
            .add_systems(Update, (
                nca_settings_ui,
                on_update_filter,
                on_update_function
            ));
    }
}

// ================================== Events ==================================== //

/// Event sent when an NCA filter was changed in the UI.
#[derive(Event, Debug)]
struct FilterChanged;

/// Event sent when an NCA activation function was changed in the UI.
#[derive(Event, Debug)]
struct FunctionChanged;

// ================================== Systems =================================== //

/// A system to ...
fn nca_settings_ui(
    mut contexts: EguiContexts,
    mut params: ResMut<NCASettings>,
    presets: Res<NCAPresets>,
    mut ev_writer_safe_filter: EventWriter<AddPresetFilter>,
    mut ev_writer_safe_fn: EventWriter<AddPresetFn>,
    mut ev_writer_filter_changed: EventWriter<FilterChanged>,
    mut ev_writer_function_changed: EventWriter<FunctionChanged>,
    mut ev_writer_reinitialize: EventWriter<Reinitialize>,
) {
    egui::Window::new("NCA Settings").show(contexts.ctx_mut(), |ui| {
        ui.spacing_mut().interact_size = bevy_egui::egui::Vec2::new(50., 20.);
        egui::Grid::new("Main Grid")
            .num_columns(1)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                for i in 0..3 {
                    let (channel, label) = match i {
                        0 => (&mut params.red, "Red Channel"),
                        1 => (&mut params.green, "Green Channel"),
                        2 => (&mut params.blue, "Blue Channel"),
                        _ => unreachable!(),
                    };
                    channel_ui(
                        ui,
                        channel,
                        label,
                        &presets,
                        &mut ev_writer_safe_filter,
                        &mut ev_writer_safe_fn,
                        &mut ev_writer_filter_changed,
                        &mut ev_writer_function_changed,
                    );
                    ui.end_row();
                }
                
            });
        ui.separator();
        if ui.button("Reinitialize").clicked() {
            ev_writer_reinitialize.send(Reinitialize);
        }
    });
}

/// A system thats triggered by the FilterChanged event. If a filter is
/// changed in the UI, the NCA control is informed via the UpdateFilter
/// event and the settings file is updated.
fn on_update_filter(
    mut ev_reader_fitler_changed: EventReader<FilterChanged>,
    mut ev_writer_save_settings: EventWriter<SaveSettings>,
    mut ev_writer_update_filter: EventWriter<UpdateFilter>,
) {
    for _ in ev_reader_fitler_changed.read() {
        ev_writer_update_filter.send(UpdateFilter);
        ev_writer_save_settings.send(SaveSettings);
    }
}

/// A system thats triggered by the FunctionChanged event. If an activation
/// function is changed in the UI, the NCA control is informed via the
/// UpdateActivationFunction event and the settings file is updated.
fn on_update_function(
    mut ev_reader_function_changed: EventReader<FunctionChanged>,
    mut ev_writer_save_settings: EventWriter<SaveSettings>,
    mut ev_writer_update_activation_function: EventWriter<UpdateActivationFunction>,
) {
    for _ in ev_reader_function_changed.read() {
        info!("Updating NCA activation functions.");
        ev_writer_update_activation_function.send(UpdateActivationFunction);
        ev_writer_save_settings.send(SaveSettings);
    }
}

// =================================== Utils ==================================== //

fn channel_ui(
    ui: &mut bevy_egui::egui::Ui,
    channel: &mut NCAChannel,
    label: &str,
    presets: &Res<NCAPresets>,
    ev_writer_safe_filter: &mut EventWriter<AddPresetFilter>,
    ev_writer_safe_fn: &mut EventWriter<AddPresetFn>,
    ev_writer_filter_changed: &mut EventWriter<FilterChanged>,
    ev_writer_function_changed: &mut EventWriter<FunctionChanged>,
) {
    egui::CollapsingHeader::new(label).show(ui, |ui| {
        ui.heading(label);
        fitler_ui(
            ui,
            &mut channel.filter,
            label,
            presets,
            ev_writer_filter_changed,
            ev_writer_safe_filter
        );
        activation_fn_ui(
            ui,
            &mut channel.activation_fn,
            label,
            presets,
            ev_writer_function_changed,
            ev_writer_safe_fn,
        );
    });
}

fn activation_fn_ui(
    ui: &mut bevy_egui::egui::Ui,
    activation_fn: &mut String,
    label: &str,
    presets: &Res<NCAPresets>,
    ev_writer_function_changed: &mut EventWriter<FunctionChanged>,
    ev_writer_safe_fn: &mut EventWriter<AddPresetFn>,

) {
    ui
        .text_edit_multiline(activation_fn)
        .changed()
        .then(|| ev_writer_function_changed.send(FunctionChanged));

    ui.horizontal(|ui| {
        if ui.button("Safe As Preset").clicked() {
            let name_option = dialog::Input::new("Please enter preset name")
                .title("Preset Name")
                .show()
                .expect("Couldn't display dialog box.");
            if let Some(name) = name_option {
                ev_writer_safe_fn.send(AddPresetFn {
                    name_and_function: (name, activation_fn.clone())
                });
            } else {
                info!("Cancelled saving activation function preset.");
            }
            
        }
        let mut selected: Option<String> = None;
        egui::ComboBox::from_id_source(label.to_owned() + " Function Preset Box")
            .selected_text("Load Preset")
            .show_ui(ui, |ui| {
                for name_and_fn in presets.activation_fn_presets() {
                    ui.selectable_value(&mut selected, Some(name_and_fn.1.clone()), name_and_fn.0.clone());
                }
            });
        if let Some(preset_fn) = selected {
            *activation_fn = preset_fn;
            ev_writer_function_changed.send(FunctionChanged);
        }
    });
}

/// System to ...
fn fitler_ui(
    ui: &mut bevy_egui::egui::Ui,
    filter: &mut Mat3,
    label: &str,
    presets: &Res<NCAPresets>,
    ev_writer_filter_changed: &mut EventWriter<FilterChanged>,
    ev_writer_safe_filter: &mut EventWriter<AddPresetFilter>,
) {
    let mut flag = false;
    egui::Grid::new(label.to_owned() + " Grid")
        .num_columns(3)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            for j in 0..3 {
                for k in 0..3 {
                    
                    ui
                        .add(egui::DragValue::new(
                            &mut filter.col_mut(j)[k],
                        ).speed(0.002))
                        .changed()
                        .then(|| flag = true );
                }
                ui.end_row();
            }
        });
    if flag {
        ev_writer_filter_changed.send(FilterChanged);
    }

    ui.horizontal(|ui| {
        if ui.button("Safe As Preset").clicked() {
            let name_option = dialog::Input::new("Please enter preset name")
                .title("Preset Name")
                .show()
                .expect("Couldn't display dialog box.");
            if let Some(name) = name_option {
                ev_writer_safe_filter.send(AddPresetFilter {
                    name_and_filter: (name, mat3_to_array(*filter))
                });
            } else {
                info!("Cancelled saving filter preset.");
            }
        }
        let mut selected: Option<[f32; 9]> = None;
        egui::ComboBox::from_id_source(label.to_owned() + " Filter Preset Box")
            .selected_text("Load Preset")
            .show_ui(ui, |ui| {
                for name_and_filter in presets.filter_presets() {
                    ui.selectable_value(&mut selected, Some(name_and_filter.1), name_and_filter.0.clone());
                }
            });
        
        if let Some(preset_filter) = selected {
            *filter = array_to_mat3(preset_filter);
            ev_writer_filter_changed.send(FilterChanged);
        }
    });
}