//! NCA control

pub mod presets;
pub mod settings;

use bevy::{prelude::*, render::renderer::RenderDevice};
use settings::NCASettings;
use std::fs::write;

use crate::SHADER_ASSET_PATH;
use super::{
    pipeline::draw::NCADrawSettings,
    NCABuffers,
    ReinitPipeline,
    create_uniform_buffer,
    utils::mat3_to_buffer_array,
};

// =================================== Plugin =================================== //

/// A plugin to control the neural cellular automaton.
pub(super) struct NCAControlPlugin;

impl Plugin for NCAControlPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                settings::SettingsPlugin,
                presets::PresetPlugin
            ))
            .add_event::<Reinitialize>()
            .add_event::<UpdateActivationFunction>()
            .add_event::<UpdateFilter>()
            .init_resource::<NCADrawSettings>()
            .add_systems(Update, (
                on_update_activation_fn,
                on_update_filter,
                on_reinitialize,
            ));
    }
}

// ================================== Events ==================================== //

/// An event to reinitialize the NCA.
#[derive(Event, Debug)]
pub struct Reinitialize;

/// An event to update the NCA's activation functions.
#[derive(Event, Debug)]
pub struct UpdateActivationFunction;

/// An event to update the NCA's filters.
#[derive(Event, Debug)]
pub struct UpdateFilter;

// ================================== Systems =================================== //

/// A system triggered by the Reinitialize event. Reinitializes the NCA.
fn on_reinitialize(
    mut ev_reader_update_filter: EventReader<Reinitialize>,
    mut reinit_res: ResMut<ReinitPipeline>,
) {
    for _ in ev_reader_update_filter.read() {
        info!("Reinitializing nca rendering pipeline.");
        reinit_res.reinit = true;
    }
}

/// A system triggered by the UpdateActivationFunction event. Rewrites the shader
/// file to contain the current activation functions, reloads the asset server and
/// sets the flag to reinitialize the render graph node of the NCA.
fn on_update_activation_fn(
    mut ev_reader_update_filter: EventReader<UpdateActivationFunction>,
    asset_server: Res<AssetServer>,
    params: ResMut<NCASettings>,
    mut reinit_res: ResMut<ReinitPipeline>,
) {
    for _ in ev_reader_update_filter.read() {
        reinit_res.reinit = true;

        info!("Writing nca shader.");
        write_shader(&params);

        info!("Reloading shader asset.");
        asset_server.reload(SHADER_ASSET_PATH);
    }
}

/// A system triggered by the UpdateFilter event. Writes the current filters to
/// the uniform buffer to pass the data to the shader.
fn on_update_filter(
    mut ev_reader_update_filter: EventReader<UpdateFilter>,
    render_device: Res<RenderDevice>,
    mut buffers: ResMut<NCABuffers>,
    params: Res<NCASettings>,
) {
    for _ in ev_reader_update_filter.read() {
        info!("Writing nca filter buffers.");
        buffers.buffer_red = 
        create_uniform_buffer(
            &render_device,
            &[mat3_to_buffer_array(params.red.filter)],
            Some("Red Uniform"),
        );
        buffers.buffer_green = 
        create_uniform_buffer(
            &render_device,
            &[mat3_to_buffer_array(params.green.filter)],
            Some("Green Uniform"),
        );
        buffers.buffer_blue = 
        create_uniform_buffer(
            &render_device,
            &[mat3_to_buffer_array(params.blue.filter)],
            Some("Blue Uniform"),
        );
    }
}

// =================================== Utils ==================================== //

/// Helper function to write the shader file.
pub fn write_shader(
    params: &NCASettings,
) {
    write(
        "assets/".to_owned() + SHADER_ASSET_PATH,
        "@group(0) @binding(0)
var texture_in: texture_storage_2d<rgba8unorm, read>;

@group(0) @binding(1)
var texture_out: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(2)
var<uniform> filter_red: mat3x3f;
@group(0) @binding(3)
var<uniform> filter_green: mat3x3f;
@group(0) @binding(4)
var<uniform> filter_blue: mat3x3f;

fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    return state;
}

fn randomFloat(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}

@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let loc = vec2<i32>(invocation_id.xy);
    let dims = textureDimensions(texture_in);
    let total_pixels = dims.x * dims.y;

    let random_red = randomFloat(invocation_id.y * dims.x + invocation_id.x);
    let random_green = randomFloat(total_pixels + invocation_id.y * dims.x + invocation_id.x);
    let random_blue = randomFloat(u32(2) * total_pixels + invocation_id.y * dims.x + invocation_id.x);
    let color = vec4<f32>(random_red, random_green, random_blue, 1.0);

    textureStore(texture_out, loc, color);
}

fn get_cell(loc: vec2<i32>, offset_x: i32, offset_y: i32) -> vec3<f32> {
    let dims = vec2<i32>(textureDimensions(texture_in));
    var offset_loc = (loc + vec2<i32>(offset_x, offset_y) + dims) % dims;
    let value: vec4<f32> = textureLoad(texture_in, offset_loc);
    return value.xyz;
}

fn nca_step(loc: vec2<i32>) -> vec3<f32> {
    var new_val = vec3<f32>(0., 0., 0.);
    for (var i: i32 = -1; i <= 1; i++) {
        for (var j: i32 = -1; j <= 1; j++) {
            new_val[0] += get_cell(loc, i, j)[0] * filter_red[i+1][j+1];
            new_val[1] += get_cell(loc, i, j)[1] * filter_green[i+1][j+1];
            new_val[2] += get_cell(loc, i, j)[2] * filter_blue[i+1][j+1];
        }
    }
    return new_val;
}

fn activation_fn_red(x: f32) -> f32 {\n\t".to_owned()
+ &params.red.activation_fn.to_owned() +
"\n}

fn activation_fn_green(x: f32) -> f32 {\n\t"
+ &params.green.activation_fn.to_owned() +
"\n}

fn activation_fn_blue(x: f32) -> f32 {\n\t"
+ &params.blue.activation_fn.to_owned() +
"\n}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let loc = vec2<i32>(invocation_id.xy);
    let val = nca_step(loc);
    let color = vec4<f32>(
        clamp(activation_fn_red(val[0]), 0., 1.),
        clamp(activation_fn_green(val[1]), 0., 1.),
        clamp(activation_fn_blue(val[2]), 0., 1.),
        1.,
    );
    textureStore(texture_out, loc, color);
}\n"
    ).expect("Couldn't write shader.");
}