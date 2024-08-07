//! NCA settings

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

use super::{UpdateActivationFunction, UpdateFilter};

// =================================== Plugin =================================== //

/// A plugin that manages any presets for the NCA.
pub(super) struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<NCASettings>()
            .add_event::<LoadSettings>()
            .add_event::<SaveSettings>()
            .add_systems(Startup, setup)
            .add_systems(Update, (
                on_load_settings,
                on_save_settings,
            ));
    }
}

// ================================ Resources =================================== //

/// A struct to hold all relevant data to run the NCA on a single color channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NCAChannel {
    /// A 3x3-matrix representing the filter of the NCA.
    pub filter: Mat3,
    /// The activation function as WGSL code.
    pub activation_fn: String,
}

impl Default for NCAChannel {
    fn default() -> Self {
        Self {
            filter: Mat3::IDENTITY,
            activation_fn: "return x;".to_string()
        }
    }
}

/// A resource holding all relevant data to run the NCA on all three color channels.
#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct NCASettings {
    pub red: NCAChannel,
    pub green: NCAChannel,
    pub blue: NCAChannel,
}

// ================================== Events ==================================== //

/// An event that triggers adding a new filter preset to the available presets.
#[derive(Event, Debug)]
pub struct LoadSettings;

/// An event that triggers adding a new actiovation function preset to the available
/// presets.
#[derive(Event, Debug)]
pub struct SaveSettings;

// ================================== Systems =================================== //

/// On startup, this system loads the NCA settings from a JSON file.
fn setup(
    mut ev_writer_update_fn: EventWriter<UpdateActivationFunction>,
    mut ev_writer_update_filter: EventWriter<UpdateFilter>,
    mut settings: ResMut<NCASettings>,
) {
    *settings = read_settings(String::from("settings.json"));
    ev_writer_update_fn.send(UpdateActivationFunction);
    ev_writer_update_filter.send(UpdateFilter);
}

/// System triggered by the LoadSettings event. Reads the JSON file containing the
/// settings and updates the NCASettings resource accordingly.
fn on_load_settings (
    mut ev_reader_load_settings: EventReader<LoadSettings>,
    mut settings: ResMut<NCASettings>,
) {
    for _ in ev_reader_load_settings.read() {
        *settings = read_settings(String::from("settings.json"));
    }
}

/// System triggered by the SaveSettings event. Saves the current settings from the
/// NCASettings resource to a JSON file.
fn on_save_settings (
    mut ev_reader_save_settings: EventReader<SaveSettings>,
    settings: Res<NCASettings>,
) {
    for _ in ev_reader_save_settings.read() {
        write_settings(String::from("settings.json"), &settings);
    }
}

// =================================== Utils ==================================== //

/// Tries to load NCA settings from the specified file path. Returns the obtained
/// settings if loading is successful, returns default settings otherwise.
pub fn read_settings(path: String) -> NCASettings {
    info!("Reading settings.");
    let contents_res = fs::read_to_string(path.clone());
    if let Ok(contents) = contents_res {
        let settings_res = serde_json::from_str::<NCASettings>(&contents);
        if let Ok(settings) = settings_res {
            settings
        } else {
            info!("Failed to parse settings, returning default value instead.");
            let settings = NCASettings::default();
            write_settings(path, &NCASettings::default());
            settings
        }
    } else {
        info!("Failed to read settings file, returning default value instead.");
        let settings = NCASettings::default();
        write_settings(path, &NCASettings::default());
        settings
    }
}

/// Tries to write the NCA settings to a specified file path. Panics if writing
/// fails.
pub fn write_settings(path: String, settings: &NCASettings) {
    info!("Writing settings.");
    fs::write(
        path,
        serde_json::to_string_pretty(settings)
            .expect("Couldn't deserialize settings."))
            .expect("Could not write to file.");
}