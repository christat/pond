#![cfg_attr(target_arch = "spirv", no_std)]

#[cfg(not(target_arch = "spirv"))]
use bytemuck::cast;

use spirv_std::glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};

#[cfg_attr(not(target_arch = "spirv"), derive(Clone, Copy))]
#[repr(C)]
pub struct PushConstants {
    pub transform: Mat4,
    pub vertex_buffer_address: u64,
}

#[cfg(not(target_arch = "spirv"))]
impl PushConstants {
    pub fn transform(mut self, transform: Mat4) -> Self {
        self.transform = transform;
        self
    }

    pub fn vertex_buffer_address(mut self, address: u64) -> Self {
        self.vertex_buffer_address = address;
        self
    }

    // TODO this is bad, and you should feel bad
    pub fn as_buffer(&self) -> [u8; 72] {
        [
            cast::<[f32; 2], [u8; 8]>(self.transform.col(0).xy().to_array()),
            cast::<[f32; 2], [u8; 8]>(self.transform.col(0).zw().to_array()),
            cast::<[f32; 2], [u8; 8]>(self.transform.col(1).xy().to_array()),
            cast::<[f32; 2], [u8; 8]>(self.transform.col(1).zw().to_array()),
            cast::<[f32; 2], [u8; 8]>(self.transform.col(2).xy().to_array()),
            cast::<[f32; 2], [u8; 8]>(self.transform.col(2).zw().to_array()),
            cast::<[f32; 2], [u8; 8]>(self.transform.col(3).xy().to_array()),
            cast::<[f32; 2], [u8; 8]>(self.transform.col(3).zw().to_array()),
            cast::<u64, [u8; 8]>(self.vertex_buffer_address),
        ]
        .as_flattened()
        .try_into()
        .unwrap()
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Default for PushConstants {
    fn default() -> Self {
        Self {
            transform: Mat4::IDENTITY,
            vertex_buffer_address: Default::default(),
        }
    }
}

#[cfg_attr(not(target_arch = "spirv"), derive(Default, Clone, Copy))]
#[repr(C)]
pub struct Vertex {
    pub position_uv_x: Vec4,
    pub normal_uv_y: Vec4,
    pub color: Vec4,
}

#[cfg(not(target_arch = "spirv"))]
impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, uv: Vec2, color: Vec4) -> Self {
        Self {
            position_uv_x: Vec4::from((position, uv.x)),
            normal_uv_y: Vec4::from((normal, uv.y)),
            color,
        }
    }
}

#[cfg(not(target_arch = "spirv"))]
pub const VERTEX_SIZE: u64 = size_of::<Vertex>() as u64;

#[cfg(not(target_arch = "spirv"))]
pub const PUSH_CONSTANTS_SIZE: u64 = size_of::<PushConstants>() as u64;
