//! Input management

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    render::extract_resource::{ExtractResource, ExtractResourcePlugin},
};
use bevy_egui::EguiContexts;

use super::pipeline::draw::NCADrawSettings;

// =================================== Plugin =================================== //

/// A plugin to manage user input. Tracks the users mouse movement and passes the
/// information to the shader for drawing on screen.
pub(super) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<NCAMouseParams>()
            .add_plugins(ExtractResourcePlugin::<NCAMouseParams>::default())
            .add_systems(Update, update_input_state);
    }
}

// ================================ Resources =================================== //

/// A resource to hold relevant information 
#[derive(Resource, ExtractResource, Clone)]
pub(super) struct NCAMouseParams {
    /// True if drawing is enabled, false otherwise.
    pub is_drawing: bool,
    /// The current mouse position in the coordinate system of the canvas.
    pub mouse_pos: Vec2,
    /// The previous mouse position in the coordinate system of the canvas.
    pub prev_mouse_pos: Vec2,

}

impl Default for NCAMouseParams {
    fn default() -> Self {
        Self {
            is_drawing: false,
            mouse_pos: Vec2::ZERO,
            prev_mouse_pos: Vec2::ZERO,
        }
    }
}

// ================================== Systems =================================== //

/// A system to react to user inputs other than interacting with the UI.
fn update_input_state(
    mut contexts: EguiContexts,
    window_query: Query<&Window>,
    mut input_state: ResMut<NCAMouseParams>,
    mut params: ResMut<NCADrawSettings>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
) {
    let Ok(primary_window) = window_query.get_single() else { return };
    let Ok((camera, camera_transform)) = camera_q.get_single() else { return };

    let ctx = contexts.ctx_mut();
    if ctx.wants_pointer_input()
        || ctx.is_pointer_over_area()
        || ctx.is_using_pointer()
        || ctx.wants_pointer_input()
    {
        params.is_drawing = false;
        return;
    } else {
        params.is_drawing = true;
    }

    for event in mouse_button_input_events.read() {
        if event.button == MouseButton::Left {
            input_state.is_drawing = event.state == ButtonState::Pressed;
        }
    }
    
    if let Some(world_position) = primary_window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        input_state.prev_mouse_pos = input_state.mouse_pos;
        input_state.mouse_pos =
            world_pos_to_canvas_pos(world_position * Vec2::new(1.0, -1.0));
    }
}

// =================================== Utils ==================================== //

/// Helper function to translate the world position from the cursor to a canvas
/// position to be used be the draw shader.
fn world_pos_to_canvas_pos(world_pos: Vec2) -> Vec2 {
    world_pos
        + Vec2::new(
            crate::SIM_SIZE.0 as f32 / 2.0,
            crate::SIM_SIZE.1 as f32 / 2.0,
        )
}