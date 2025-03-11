#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{Vec3, Vec4},
    spirv,
};

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vertex_index: i32,
    #[spirv(position)] out_pos: &mut Vec4,
    out_color: &mut Vec3,
) {
    let vertices = [
        Vec3::new(0.6, 0.75, 0.0),
        Vec3::new(-0.6, 0.75, 0.0),
        Vec3::new(0.0, -0.75, 0.0),
    ];

    let colors = [
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ];

    *out_pos = Vec4::from((vertices[vertex_index as usize], 1.0));
    *out_color = colors[vertex_index as usize];
}

#[spirv(fragment)]
pub fn main_fs(in_color: Vec3, output: &mut Vec4) {
    *output = Vec4::from((in_color, 1.0));
}
