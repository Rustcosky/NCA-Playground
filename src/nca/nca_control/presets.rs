//! NCA presets.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

// =================================== Plugin =================================== //

/// A plugin that manages any presets for the NCA.
pub(super) struct PresetPlugin;

impl Plugin for PresetPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<NCAPresets>()
            .add_event::<AddPresetFilter>()
            .add_event::<AddPresetFn>()
            .add_systems(Startup, setup)
            .add_systems(Update, (on_safe_preset_filter, on_safe_preset_fn));
    }
}

// ================================ Resources =================================== //

/// A resource holding all available presets.
#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct NCAPresets {
    filter_presets: Vec<(String, [f32; 9])>,
    activation_fn_presets: Vec<(String, String)>,
}

impl NCAPresets {
    /// Returns a vector of available presets for NCA filters, each as a tuple with
    /// the name of the preset in the 0th component, and ...
    pub fn filter_presets(&self) -> &Vec<(String, [f32; 9])> {
        &self.filter_presets
    }

    /// Returns a vector of available presets for NCA activation functions, each as
    /// a tuple with the name of the preset in the 0th component and the activation
    /// function as a String containing WGSL code in the 1st component.
    pub fn activation_fn_presets(&self) -> &Vec<(String, String)> {
        &self.activation_fn_presets
    }
}

// ================================== Events ==================================== //

/// An event that triggers adding a new filter preset to the available presets.
#[derive(Event, Debug)]
pub struct AddPresetFilter {
    pub name_and_filter: (String, [f32; 9]),
}

/// An event that triggers adding a new actiovation function preset to the available
/// presets.
#[derive(Event, Debug)]
pub struct AddPresetFn {
    pub name_and_function: (String, String),
}

// ================================== Systems =================================== //

/// On startup, this system loads the available presets from a JSON-file.
fn setup(
    mut presets: ResMut<NCAPresets>,
) {
    *presets = read_presets(String::from("presets.json"));
}

/// System triggered by the AddPresetFilter event. Adds the events contents as a new
/// available filter preset and writes the resulting available presets to the preset
/// file.
fn on_safe_preset_filter (
    mut ev_reader_safe_fitler: EventReader<AddPresetFilter>,
    mut presets: ResMut<NCAPresets>,
) {
    for event in ev_reader_safe_fitler.read() {
        presets.filter_presets.push(event.name_and_filter.clone());
        write_presets(String::from("presets.json"), &presets);
    }
}

/// System triggered by the AddPresetFn event. Adds the events contents as a new
/// available activation function preset and writes the resulting available presets
/// to the preset file.
fn on_safe_preset_fn (
    mut ev_reader_safe_fn: EventReader<AddPresetFn>,
    mut presets: ResMut<NCAPresets>,
) {
    for event in ev_reader_safe_fn.read() {
        presets.activation_fn_presets.push(event.name_and_function.clone());
        write_presets(String::from("presets.json"), &presets);
    }
}

// =================================== Utils ==================================== //

/// Tries to load presets from the specified file path. Returns the obtained presets
/// if loading is successful, returns empty presets otherwise.
fn read_presets(path: String) -> NCAPresets {
    info!("Reading presets.");
    let contents_res = fs::read_to_string(path.clone());
    if let Ok(contents) = contents_res {
        let presets_res = serde_json::from_str::<NCAPresets>(&contents);
        if let Ok(presets) = presets_res {
            presets
        } else {
            info!("Failed to parse presets, returning default value instead.");
            let presets = NCAPresets::default();
            write_presets(path, &NCAPresets::default());
            presets
        }
    } else {
        info!("Failed to read preset file, returning default value instead.");
        let presets = NCAPresets::default();
        write_presets(path, &NCAPresets::default());
        presets
    }
}

/// Tries to write the presets to a specified file path. Panics if writing fails.
fn write_presets(path: String, presets: &NCAPresets) {
    info!("Writing presets.");
    fs::write(
        path,
        serde_json::to_string_pretty(presets)
            .expect("Couldn't deserialize settings."))
            .expect("Could not write to file.");
}