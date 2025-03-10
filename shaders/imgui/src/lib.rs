#![no_std]

use spirv_std::glam::{Vec4, Vec2, IVec2};
use spirv_std::image::Image2d;
use spirv_std::spirv;

pub struct Shared {
    color: Vec4,
    uv: IVec2,
}

#[spirv(fragment)]
pub fn main_fs(
    output: &mut Vec4,
    #[spirv(descriptor_set = 0, binding = 0)] texture: &Image2d,
    #[spirv(flat)] input: Shared,
) {
    *output = input.color * texture.fetch(input.uv);
}

pub struct PushConstants {
    scale: Vec2,
    translate: Vec2
}

#[spirv(vertex)]
pub fn main_vs(
    push_constants: PushConstants,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] position: &Vec2,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] uv: &IVec2,
    #[spirv(uniform, descriptor_set = 0, binding = 2)] color: &Vec4,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
    #[spirv(flat)] output: &mut Shared
) {
    output.color = *color;
    output.uv = *uv;
    *out_pos = Vec4::from((*position * push_constants.scale + push_constants.translate, 0.0, 1.0));
}