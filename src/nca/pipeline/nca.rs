//! The NCA rendering pipeline

use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
        Render,
        RenderSet,
    },
};
use std::borrow::Cow;

use crate::{SHADER_ASSET_PATH, SIM_SIZE, WORKGROUP_SIZE};
use super::super::{NCABuffers, ReinitPipeline};

// ================================= Constants ================================== //

/// Holds the NCA filter data for writing to the shader buffer.
/// 
/// Due to memory alignment requirements of wgsl, each channel holds 12 float,
/// even though a filter is a 3x3-matrix with only 9 entries. ...
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct NCAFilter {
    pub red: [f32; 12],
    pub green: [f32; 12],
    pub blue: [f32; 12],
}

impl NCAFilter {
    pub fn empty() -> Self {
        Self {
            red: [0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.],
            green: [0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.],
            blue: [0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.],
        }
    }
}

// =================================== Plugin =================================== //

/// A plugin that manages the NCA rendering pipeline.
pub struct NCAPipelinePlugin;

impl Plugin for NCAPipelinePlugin {
    fn build(&self, render_app: &mut App) {
        render_app
            .add_systems(Render, queue_nca_bind_group.in_set(RenderSet::Queue));
    }
}

// ================================== Pipeline ================================== //

/// A resource holding the rendering pipeline data for the NCA.
#[derive(Resource)]
pub struct NCAPipeline {
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    texture_bind_group_layout: BindGroupLayout,
}

impl FromWorld for NCAPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        
        let texture_bind_group_layout = render_device.create_bind_group_layout(
            Some("NCA Bind Group Layout"),
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                filter_layout_entry(2),
                filter_layout_entry(3),
                filter_layout_entry(4),
            ],
        );

        let shader = world.load_asset(SHADER_ASSET_PATH);
        let pipeline_cache = world.resource::<PipelineCache>();

        let init_pipeline = pipeline_cache.queue_compute_pipeline(
            ComputePipelineDescriptor {
                label: Some(std::borrow::Cow::Borrowed("NCA Init Pipeline")),
                layout: vec![texture_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("init"),
            }
        );
        let update_pipeline = pipeline_cache.queue_compute_pipeline(
            ComputePipelineDescriptor {
                label: Some(std::borrow::Cow::Borrowed("NCA Update Pipeline")),
                layout: vec![texture_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader,
                shader_defs: vec![],
                entry_point: Cow::from("update"),
            }
        );

        Self {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}

// ================================== BindGroup ================================== //

/// An asset holding the image handles to the two textures.
#[derive(Asset, Resource, ExtractResource, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct NCAImages{
    pub texture_a: Handle<Image>,
    pub texture_b: Handle<Image>,
}

/// A resource holding the two bind groups corresponding to the two textures.
#[derive(Resource)]
pub struct NCABindGroup(pub [BindGroup; 2]);

fn queue_nca_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    buffers: Res<NCABuffers>,
    pipeline: Res<NCAPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    game_of_life_image: Res<NCAImages>,
) {
    let view_a = gpu_images.get(&game_of_life_image.texture_a).unwrap();
    let view_b = gpu_images.get(&game_of_life_image.texture_b).unwrap();
    let bind_group_0 = render_device.create_bind_group(
        None,
        &pipeline.texture_bind_group_layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view_a.texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&view_b.texture_view),
            },
            filter_bind_group_entry(2, &buffers.buffer_red),
            filter_bind_group_entry(3, &buffers.buffer_green),
            filter_bind_group_entry(4, &buffers.buffer_blue),
        ],
    );
    let bind_group_1 = render_device.create_bind_group(
        None,
        &pipeline.texture_bind_group_layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view_b.texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&view_a.texture_view),
            },
            filter_bind_group_entry(2, &buffers.buffer_red),
            filter_bind_group_entry(3, &buffers.buffer_green),
            filter_bind_group_entry(4, &buffers.buffer_blue),
        ],
    );
    commands.insert_resource(NCABindGroup([bind_group_0, bind_group_1]));
}

// ================================== Nodes ================================== //

/// A label for the node in the rendering graph corresponding to the NCA.
#[derive(RenderLabel, Debug, PartialEq, Eq, Clone, Hash)]
pub struct NCALabel;

/// The state of the NCA node.
#[derive(Debug, Default, PartialEq)]
enum NCAState {
    #[default]
    Loading,
    Init,
    Update(usize),
}

/// The NCA node in the rendering graph.
#[derive(Debug, Default)]
pub(super) struct NCANode {
    state: NCAState,
}

impl Node for NCANode {
    fn update(&mut self, world: &mut World) {
        
        let reinit = world.resource::<ReinitPipeline>().reinit;
        if reinit {
            info!("Reinitializing NCA pipeline.");
            world.init_resource::<NCAPipeline>();
            self.state = NCAState::Loading;
        }

        let reinit = &mut world.resource_mut::<ReinitPipeline>().reinit;
        *reinit = false;

        let pipeline = world.resource::<NCAPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        match self.state {
            NCAState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    info!("Initialized NCA pipeline.");
                    self.state = NCAState::Init;
                }
            }
            NCAState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    info!("Updated NCA pipeline from initial state.");
                    self.state = NCAState::Update(1);
                }
            }
            NCAState::Update(0) => {
                self.state = NCAState::Update(1);
            }
            NCAState::Update(1) => {
                self.state = NCAState::Update(0);
            }
            NCAState::Update(_) => unreachable!(),
        }
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let reinit = &world.resource::<ReinitPipeline>().reinit;
        if *reinit {
            info!("Running, but is paused.");
            return Ok(());
        }

        let texture_bind_group = &world.resource::<NCABindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<NCAPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        match self.state {
            NCAState::Loading => {}
            NCAState::Init => {
                if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.init_pipeline) {
                    pass.set_bind_group(0, &texture_bind_group[0], &[]);
                    pass.set_pipeline(init_pipeline);
                    pass.dispatch_workgroups(
                        SIM_SIZE.0 / WORKGROUP_SIZE,
                        SIM_SIZE.1 / WORKGROUP_SIZE,
                        1,
                    );
                } else {
                    return Ok(());
                }
            }
            NCAState::Update(index) => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_bind_group(0, &texture_bind_group[index], &[]);
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(
                    SIM_SIZE.0 / WORKGROUP_SIZE,
                    SIM_SIZE.1 / WORKGROUP_SIZE,
                    1,
                );
            }
        }

        Ok(())
    }
}

// =================================== Utils ==================================== //

/// Creates a BindGroupEntry for one NCA filter for passing to the shader.
fn filter_bind_group_entry(binding: u32, buffer: &Buffer) -> BindGroupEntry<'_> {
    BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

/// Creates a BindGroupLayoutEntry for one NCA filter for passing to the shader.
fn filter_layout_entry(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer { 
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: BufferSize::new(
                (std::mem::size_of::<NCAFilter>() / 3) as _,
            ),
        },
        count: None,
    }
}