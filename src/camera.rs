//! Camera functionalities

use bevy::{input::mouse::{MouseScrollUnit, MouseWheel}, prelude::*};

// ================================= Constants ================================== //

/// Movement speed of the camera.
const CAMERA_MOVE_SPEED: f32 = 500.0;

// =================================== Plugin =================================== //

/// A plugin to manage the camera.
pub(super) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_controller);
    }
}

// ================================== Systems =================================== //

/// A system for camera control.
/// 
/// The camera can be moved around by using WASD. The mouse wheel can be used to
/// zoom in and out.
fn camera_controller(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) {
    for (mut transform, mut ortho) in query.iter_mut() {
        let up = keys.pressed(KeyCode::KeyW);
        let down = keys.pressed(KeyCode::KeyS);
        let left = keys.pressed(KeyCode::KeyA);
        let right = keys.pressed(KeyCode::KeyD);

        let x_axis = right as i8 - left as i8;
        let y_axis = up as i8 - down as i8;
        let mut move_delta = Vec2::new(x_axis as f32, y_axis as f32);

        if move_delta != Vec2::ZERO {
            move_delta /= move_delta.length();

            let z = transform.translation.z;
            transform.translation +=
                move_delta.extend(z) * CAMERA_MOVE_SPEED * time.delta_seconds();

            transform.translation.z = z;
        }

        for event in mouse_wheel_events.read() {
            let mut x_scroll_diff = 0.0;
            let mut y_scroll_diff = 0.0;

            match event.unit {
                MouseScrollUnit::Line => {
                    x_scroll_diff += event.x;
                    y_scroll_diff += event.y;
                }
                MouseScrollUnit::Pixel => {
                    const PIXELS_PER_LINE: f32 = 38.0;

                    y_scroll_diff += event.y / PIXELS_PER_LINE;
                    x_scroll_diff += event.x / PIXELS_PER_LINE;
                }
            }

            if x_scroll_diff != 0.0 || y_scroll_diff != 0.0 {
                if y_scroll_diff < 0.0 {
                    ortho.scale *= 1.05;
                } else {
                    ortho.scale *= 1.0 / 1.05;
                }

                ortho.scale = ortho.scale.clamp(0.15, 5.);
            }
        }
    }
}