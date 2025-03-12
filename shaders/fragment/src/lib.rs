#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{Vec2, Vec3, Vec4},
    spirv,
};

#[spirv(fragment)]
pub fn main_fs(in_color: Vec3, _in_uv: Vec2, output: &mut Vec4) {
    *output = Vec4::from((in_color, 1.0));
}
