//! Neural cellular automaton

pub mod input;
pub mod nca_control;
pub mod pipeline;
pub mod ui;
pub mod utils;

use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssetUsages,
        render_resource::{
            Buffer,
            BufferInitDescriptor,
            BufferUsages,
            Extent3d,
            TextureDimension,
            TextureFormat,
            TextureUsages,
        },
        renderer::RenderDevice,
    },
};

use crate::SIM_SIZE;
use pipeline::{draw::NCADrawSettings, nca::{NCAFilter, NCAImages}};

// =================================== Plugin =================================== //

/// A plugin that manages everything related to the NCA. Contains the infrastructure
/// for the rendering pipelines, control over the NCA settings, a UI as well as
/// input management.
pub struct NCAPlugin;

impl Plugin for NCAPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ReinitPipeline>()
            .add_plugins((
                ExtractResourcePlugin::<NCABuffers>::default(),
                ExtractResourcePlugin::<NCADrawSettings>::default(),
                ExtractResourcePlugin::<NCAImages>::default(),
                ExtractResourcePlugin::<ReinitPipeline>::default(),
                input::InputPlugin,
                nca_control::NCAControlPlugin,
                pipeline::PipelinesPlugin,
                ui::UIPlugin,
            ))
            .add_systems(Startup, setup)
            .add_systems(Update, switch_textures);
    }
}

// ================================ Resources =================================== //

/// Resource with a flag to reinitialize the rendering pipeline.
#[derive(Resource, ExtractResource, Debug, Default, Clone)]
pub struct ReinitPipeline {
    pub reinit: bool,
}

/// A buffer to hold the filter data of the NCA. Is passed to the shader as a
/// uniform.
#[derive(Resource, Clone, ExtractResource)]
pub(super) struct NCABuffers {
    pub buffer_red: Buffer,
    pub buffer_green: Buffer,
    pub buffer_blue: Buffer,
}

// ================================== Systems =================================== //

/// On startup, this system adds two images (in- and output for the NCA compute
/// shader), spawns a sprite bundle corresponding to one of the images and adds
/// .
fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>
) {
    let texture_a = create_image(SIM_SIZE.0, SIM_SIZE.1);
    let texture_b = create_image(SIM_SIZE.0, SIM_SIZE.1);
    let texture_a = images.add(texture_a);
    let texture_b = images.add(texture_b);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIM_SIZE.0 as f32, SIM_SIZE.1 as f32)),
            ..default()
        },
        texture: texture_a.clone(),
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(NCAImages{ texture_a, texture_b });
    commands.insert_resource(
        create_uniform_nca_buffer(NCAFilter::empty(), &render_device)
    );
}

/// A system that switches the pointer of the displayed image after each compute
/// shader pass.
fn switch_textures(
    images: Res<NCAImages>,
    mut displayed: Query<&mut Handle<Image>>,
) {
    let mut displayed = displayed.single_mut();
    if *displayed == images.texture_a {
        *displayed = images.texture_b.clone_weak();
    } else {
        *displayed = images.texture_a.clone_weak();
    }
}

// =================================== Utils ==================================== //

fn create_image(width: u32, height: u32) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    image
}

fn create_uniform_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(
    device: &RenderDevice,
    data: &[T],
    label: Option<&str>,
) -> Buffer {
    device.create_buffer_with_data(&BufferInitDescriptor {
        label,
        contents: bytemuck::cast_slice(data),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
}

fn create_uniform_nca_buffer(
    filter: NCAFilter,
    device: &RenderDevice,
) -> NCABuffers {
    let buffer_red = create_uniform_buffer(
        device,
        &[filter.red],
        Some("Red Uniform"),
    );
    let buffer_green = create_uniform_buffer(
        device,
        &[filter.green],
        Some("Green Uniform"),
    );
    let buffer_blue = create_uniform_buffer(
        device,
        &[filter.blue],
        Some("Blue Uniform"),
    );
    NCABuffers{ buffer_red, buffer_green, buffer_blue }
}