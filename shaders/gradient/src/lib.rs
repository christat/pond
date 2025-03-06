#![no_std]

use spirv_std::spirv;
use spirv_std::image::Image;
use spirv_std::glam::{UVec2, Vec2, Vec4};

pub type Image2 = Image!(2D, format=rgba16f, sampled=false, depth=false);

#[spirv(compute(threads(16, 16)))]
pub fn main(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] image: &mut Image2,
    #[spirv(global_invocation_id)] global_coord: Vec2,
    #[spirv(local_invocation_id)] local_coord: Vec2,
) {
    let texel_coord = UVec2::new(global_coord.x as u32, global_coord.y as u32);
    let image_size: UVec2 = image.query_size();

    if texel_coord.x < image_size.x && texel_coord.y < image_size.y {
        let mut color = Vec4::new(0.0, 0.0, 0.0, 1.0);

        if local_coord.x != 0.0 && local_coord.y != 0.0 {
            color.x = texel_coord.x as f32 / image_size.x as f32;
            color.y = texel_coord.y as f32 / image_size.y as f32;
        }

        unsafe { image.write(texel_coord, color) };
    }
}