#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{Vec2, Vec4},
    image::{Image2d, SampledImage},
    spirv,
};

#[derive(Copy, Clone)]
pub struct PushConstants {
    scale: Vec2,
    translate: Vec2,
}

#[repr(C)]
pub struct Shared {
    color: Vec4,
    uv: Vec2,
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(push_constant)] constants: &PushConstants,
    in_pos: Vec2,
    in_uv: Vec2,
    in_color: Vec4,
    out_shared: &mut Shared,
    #[spirv(position)] out_pos: &mut Vec4,
) {
    out_shared.color = in_color;
    out_shared.uv = in_uv;
    *out_pos = Vec4::from((in_pos * constants.scale + constants.translate, 0.0, 1.0));
}

#[spirv(fragment)]
pub fn main_fs(
    in_shared: Shared,
    #[spirv(uniform_constant, descriptor_set = 0, binding = 0)] in_texture: &SampledImage<Image2d>,
    output: &mut Vec4,
) {
    *output = in_shared.color * in_texture.sample(in_shared.uv);
}
