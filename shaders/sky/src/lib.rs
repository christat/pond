#![no_std]

use spirv_std::glam::{UVec2, UVec3, Vec2, Vec2Swizzles, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};
use spirv_std::image::Image;
#[allow(unused)]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

pub type Image2 = Image!(2D, format = rgba16f, sampled = false, depth = false);

#[derive(Copy, Clone)]
#[allow(unused)]
pub struct PushConstants {
    data_0: Vec4,
    data_1: Vec4,
    data_2: Vec4,
    data_3: Vec4,
}

fn noise_2d(sample_pos: Vec2) -> f32 {
    let x_hash = (sample_pos.x * 37.0).cos();
    let y_hash = (sample_pos.y * 57.0).cos();
    (415.92653 * (x_hash + y_hash)).fract()
}

fn noisy_starfield(sample_pos: Vec2, threshold: f32) -> f32 {
    let value = noise_2d(sample_pos);
    match value >= threshold {
        true => ((value - threshold) / (1.0 - threshold)).powf(6.0),
        false => 0.0,
    }
}

// Stabilize noisy_starfield() by only sampling at integer values.
fn stable_starfield(sample_pos: Vec2, threshold: f32) -> f32 {
    // Linear interpolation between four samples.
    // Note: This approach has some visual artifacts.
    // There must be a better way to "anti alias" the star field.
    let fract = Vec2::new(sample_pos.x.fract(), sample_pos.y.fract());
    let floor_sample = Vec2::new(sample_pos.x.floor(), sample_pos.y.floor());
    let v1 = noisy_starfield(floor_sample, threshold);
    let v2 = noisy_starfield(floor_sample + Vec2::new(0.0, 1.0), threshold);
    let v3 = noisy_starfield(floor_sample + Vec2::new(1.0, 0.0), threshold);
    let v4 = noisy_starfield(floor_sample + Vec2::new(1.0, 1.0), threshold);

    v1 * (1.0 - fract.x) * (1.0 - fract.y)
        + v2 * (1.0 - fract.x) * fract.y
        + v3 * fract.x * (1.0 - fract.y)
        + v4 * fract.x * fract.y
}

fn sky(texel_coord: UVec2, image_size: UVec2, constants: &PushConstants) -> Vec4 {
    let mut vec_color = constants.data_0.xyz() * texel_coord.y as f32 / image_size.y as f32;

    // Note: Choose fThreshhold in the range [0.99, 0.9999].
    // Higher values (i.e., closer to one) yield a sparser starfield.
    let star_field_threshold = constants.data_1.w;

    // Stars with a slow crawl
    let star_rate = Vec2::new(0.2, -0.06);
    let sample_pos = texel_coord.xy().as_vec2() + star_rate;
    let star_val = stable_starfield(sample_pos, star_field_threshold);
    vec_color += Vec3::splat(star_val);

    Vec4::from((vec_color, 1.0))
}

#[spirv(compute(threads(16, 16)))]
pub fn main_cs(
    #[spirv(push_constant)] constants: &PushConstants,
    #[spirv(descriptor_set = 0, binding = 0)] image: &Image2,
    #[spirv(global_invocation_id)] global_coord: UVec3,
) {
    let texel_coord = global_coord.xy();
    let image_size: UVec2 = image.query_size();

    if texel_coord.x < image_size.x && texel_coord.y < image_size.y {
        unsafe { image.write(texel_coord, sky(texel_coord, image_size, constants)) };
    }
}
