#![no_std]

use spirv_std::glam::{UVec2, UVec3, Vec3Swizzles, Vec4};
use spirv_std::image::Image;
use spirv_std::spirv;

pub type Image2 = Image!(2D, format = rgba16f, sampled = false, depth = false);

#[spirv(compute(threads(16, 16)))]
pub fn main_cs_old(
    #[spirv(descriptor_set = 0, binding = 0)] image: &Image2,
    #[spirv(global_invocation_id)] global_coord: UVec3,
    #[spirv(local_invocation_id)] local_coord: UVec3,
) {
    let texel_coord = global_coord.xy();
    let image_size: UVec2 = image.query_size();

    if texel_coord.x < image_size.x && texel_coord.y < image_size.y {
        let mut color = Vec4::new(0.0, 0.0, 0.0, 1.0);

        if local_coord.x != 0 && local_coord.y != 0 {
            color.x = texel_coord.x as f32 / image_size.x as f32;
            color.y = texel_coord.y as f32 / image_size.y as f32;
        }

        unsafe { image.write(texel_coord, color) };
    }
}

#[derive(Copy, Clone)]
#[allow(unused)]
pub struct PushConstants {
    data_0: Vec4,
    data_1: Vec4,
    data_2: Vec4,
    data_3: Vec4,
}

#[spirv(compute(threads(16, 16)))]
pub fn main_cs(
    #[spirv(push_constant)] constants: &PushConstants,
    #[spirv(descriptor_set = 0, binding = 0)] image: &Image2,
    #[spirv(global_invocation_id)] global_coord: UVec3,
) {
    let texel_coord = global_coord.xy();
    let image_size: UVec2 = image.query_size();

    let top_color = constants.data_0;
    let bottom_color = constants.data_1;

    if texel_coord.x < image_size.x && texel_coord.y < image_size.y {
        let blend = texel_coord.y as f32 / image_size.y as f32;
        let color = blend * bottom_color + (1.0 - blend) * top_color;
        unsafe { image.write(texel_coord, color) };
    }
}
