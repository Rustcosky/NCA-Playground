//! Manages all rendering pipelines

pub mod draw;
pub mod nca;

use bevy::{prelude::*, render::{render_graph::RenderGraph, RenderApp}};

use draw::{NCADrawLabel, NCADrawNode, NCADrawPipeline, NCADrawPipelinePlugin};
use nca::{NCALabel, NCANode, NCAPipeline, NCAPipelinePlugin};

// =================================== Plugin =================================== //

/// A plugin to manage to manage the two rendering pipelines: for the neural cellular
/// automaton and for letting the user draw on screen.
pub(super) struct PipelinesPlugin;

impl Plugin for PipelinesPlugin {
    fn build(&self, app: &mut App) {
        // The rendering pipelines are only relevant for the rendering world. So we
        // only need to add our plugins to the rendering sub app.
        let render_app = app.sub_app_mut(RenderApp);

        // Add all pipeline plugins:
        render_app
            .add_plugins((
                NCAPipelinePlugin,
                NCADrawPipelinePlugin,
            ));
        
        // Build render graph:
        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(NCALabel, NCANode::default());
        render_graph.add_node(NCADrawLabel, NCADrawNode::default());
        render_graph.add_node_edge(NCALabel, bevy::render::graph::CameraDriverLabel);
        render_graph.add_node_edge(NCADrawLabel, bevy::render::graph::CameraDriverLabel);
    }
    
    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<NCAPipeline>();
        render_app.init_resource::<NCADrawPipeline>();
    }
}