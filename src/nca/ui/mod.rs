//! UI support

pub mod draw;
pub mod nca;

use bevy::prelude::*;

// =================================== Plugin =================================== //

/// A plugin that provides a UI (based on Bevy's EGUI integration) to interact with the
/// NCA. It comprises a UI-plugin for controlling the settings of the NCA itself and
/// another one to control the settings for drawing on the texture.
pub(super) struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                bevy_egui::EguiPlugin,
                draw::UIDrawPlugin,
                nca::UINCAPlugin,
            ));
    }
}