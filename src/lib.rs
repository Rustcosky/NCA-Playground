//! EPILEPSY WARNING: This application may potentially trigger seizures for people
//! with photosensitive epilepsy. User discretion is advised.
//! 
//! This crate provides a playground to experiment with neural cellular automata
//! (NCA).

pub mod camera;
pub mod nca;

use bevy::app::{App, Plugin};

// ================================= Constants ================================== //

/// The file path from ./assets/ to the shader for the NCA.
const SHADER_ASSET_PATH: &str = "shaders/nca.wgsl";

/// Size of the simulation in pixels.
const SIM_SIZE: (u32, u32) = (1920, 1080);

/// Size of the workgroups on the GPU for the compute shaders.
const WORKGROUP_SIZE: u32 = 8;

// =================================== Plugin =================================== //

/// Main plugin, containing the NCA functionalities, input and camera control as
/// well as a UI.
pub struct NCAPlaygroundPlugin;

impl Plugin for NCAPlaygroundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                camera::CameraPlugin,
                nca::NCAPlugin,
            ));
    }
}
