//! The rendering pipeline for drawing on screen

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

use super::{
    super::input::NCAMouseParams,
    nca::{NCABindGroup, NCAImages},
};

// =================================== Plugin =================================== //

#[derive(Resource, ExtractResource, Debug, Clone, Copy)]
pub struct NCADrawSettings {
    pub is_drawing: bool,

    pub brush_size: f32,
    pub brush_type: u32,
    pub brush_color: [f32; 3],
}

impl Default for NCADrawSettings {
    fn default() -> Self {
        Self {
            is_drawing: true,
            brush_size: 10.,
            brush_type: 0,
            brush_color: [1., 1., 1.],
        }
    }
}

/// A plugin that manages the rendering pipeline for drawing on screen.
pub(super) struct NCADrawPipelinePlugin;

impl Plugin for NCADrawPipelinePlugin {
    fn build(&self, render_app: &mut App) {
        render_app
            .add_systems(Render, queue_draw_bind_group.in_set(RenderSet::Queue));
    }
}

// ================================= Constants ================================== //

#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct NCAPushConstants {
    draw_start: [f32; 2],
    draw_end: [f32; 2],

    brush_size: f32,
    brush_type: u32,
    brush_color: [f32; 3]
}

impl NCAPushConstants {
    pub fn new(
        draw_start: Vec2,
        draw_end: Vec2,
        brush_size: f32,
        brush_type: u32,
        brush_color: [f32; 3],
    ) -> Self {
        Self {
            draw_start: draw_start.to_array(),
            draw_end: draw_end.to_array(),
            brush_size,
            brush_type,
            brush_color,
        }
    }
}

// ================================== Pipeline ================================== //

/// A resource holding the rendering pipeline data for drawing on screen.
#[derive(Resource)]
pub(super) struct NCADrawPipeline {
    draw_pipeline: CachedComputePipelineId,
    draw_bind_group_layout: BindGroupLayout,
}

impl FromWorld for NCADrawPipeline {
    fn from_world(world: &mut World) -> Self {
        let pipeline_cache = world.resource::<PipelineCache>();

        let draw_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(
                    Some("NCA Draw Bind Group Layout"),
                    &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                );

        let brush_shader = world.resource::<AssetServer>().load("shaders/draw.wgsl");

        let draw_pipeline = pipeline_cache.queue_compute_pipeline(
                ComputePipelineDescriptor {
                shader: brush_shader,
                shader_defs: vec![],
                entry_point: Cow::from("draw"),
                layout: vec![draw_bind_group_layout.clone()],
                label: Some(std::borrow::Cow::Borrowed("NCA Draw Pipeline")),
                push_constant_ranges: [PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: 0..std::mem::size_of::<NCAPushConstants>() as u32,
                }]
                .to_vec(),
            }
        );

        Self {
            draw_pipeline,
            draw_bind_group_layout,
        }
    }
}

// ================================== BindGroup ================================== //

/// A resource holding the bind groups corresponding to the texture.
#[derive(Resource)]
struct NCADrawBindGroup(BindGroup);

fn queue_draw_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<NCADrawPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    nca_images: Res<NCAImages>,
) {
    let view = &gpu_images.get(&nca_images.texture_a).unwrap();
    let draw_bind_group = render_device.create_bind_group(
        Some("NCA Draw Bind Group"),
        &pipeline.draw_bind_group_layout,
        &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view.texture_view),
        }],
    );
    commands.insert_resource(NCADrawBindGroup(draw_bind_group));
}

// ================================== Nodes ================================== //

/// A label for the node in the rendering graph corresponding to drawing on
/// screen.
#[derive(RenderLabel, Debug, PartialEq, Eq, Clone, Hash)]
pub struct NCADrawLabel;

/// The state of the draw node.
enum NCADrawState {
    Loading,
    Update,
}

/// The node for drawing on screen in the rendering graph.
pub(super) struct NCADrawNode {
    state: NCADrawState,
}

impl Default for NCADrawNode {
    fn default() -> Self {
        Self {
            state: NCADrawState::Loading,
        }
    }
}

impl Node for NCADrawNode {
    fn update(&mut self, world: &mut World) {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<NCADrawPipeline>();

        match self.state {
            NCADrawState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.draw_pipeline)
                {
                    self.state = NCADrawState::Update;
                }
            }
            NCADrawState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let params = &world.resource::<NCAMouseParams>();

        if params.is_drawing {
            let draw_params = &world.resource::<NCADrawSettings>();
            let texture_bind_group = &world.resource::<NCABindGroup>().0;
            let draw_bind_group = &world.resource::<NCADrawBindGroup>().0;
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipeline = world.resource::<NCADrawPipeline>();

            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, &texture_bind_group[1], &[]);

            match self.state {
                NCADrawState::Loading => {}
                NCADrawState::Update => {
                    let draw_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.draw_pipeline)
                        .unwrap();
                    let pc =
                        NCAPushConstants::new(
                            params.mouse_pos,
                            params.prev_mouse_pos,
                            draw_params.brush_size,
                            draw_params.brush_type,
                            draw_params.brush_color,
                        );

                    pass.set_pipeline(draw_pipeline);
                    pass.set_bind_group(0, draw_bind_group, &[]);
                    pass.set_push_constants(0, bytemuck::cast_slice(&[pc]));
                    pass.dispatch_workgroups(
                        crate::SIM_SIZE.0 / crate::WORKGROUP_SIZE,
                        crate::SIM_SIZE.1 / crate::WORKGROUP_SIZE,
                        1,
                    );
                }
            }
        }

        Ok(())
    }
}