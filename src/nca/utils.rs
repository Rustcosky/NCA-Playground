//! Matrix utils

use bevy::math::{Mat3, Vec3};

pub fn array_to_mat3(array: [f32; 9]) -> Mat3 {
    Mat3 {
        x_axis: Vec3::new(array[0], array[1], array[2]),
        y_axis: Vec3::new(array[3], array[4], array[5]),
        z_axis: Vec3::new(array[6], array[7], array[8]),
    }
}

pub fn mat3_to_array(mat: Mat3) -> [f32; 9] {
    [
        mat.x_axis[0], mat.x_axis[1], mat.x_axis[2],
        mat.y_axis[0], mat.y_axis[1], mat.y_axis[2],
        mat.z_axis[0], mat.z_axis[1], mat.z_axis[2],
    ]
}

pub fn mat3_to_buffer_array(mat: Mat3) -> [f32; 12] {
    [
        mat.x_axis[2], mat.y_axis[2], mat.z_axis[2], 0.,
        mat.x_axis[1], mat.y_axis[1], mat.z_axis[1], 0.,
        mat.x_axis[0], mat.y_axis[0], mat.z_axis[0], 0.,
    ]
}