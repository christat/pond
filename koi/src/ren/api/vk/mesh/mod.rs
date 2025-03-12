use super::{ImmediateManager, buffer::Buffer, resource_allocator::AllocatedResources};

use ash::{Device as DeviceHandle, vk};
use gpu_allocator::{MemoryLocation, vulkan as vka};
use koi_gpu::{VERTEX_SIZE, Vertex};

pub struct Mesh {
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub vertex_buffer_address: vk::DeviceAddress,
}

pub const INDEX_SIZE: u64 = size_of::<u32>() as u64;

impl Mesh {
    pub fn new(
        device_handle: &DeviceHandle,
        allocator: &mut vka::Allocator,
        resources: &mut AllocatedResources,
        immediate_manager: &mut ImmediateManager,
        indices: &[u32],
        vertices: &[Vertex],
    ) -> Self {
        let index_buffer_size = indices.len() as u64 * INDEX_SIZE;
        let index_buffer = Buffer::new(
            device_handle,
            allocator,
            resources,
            index_buffer_size,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            "mesh_indices",
            MemoryLocation::GpuOnly,
        );

        let vertex_buffer_size = vertices.len() as u64 * VERTEX_SIZE;
        let vertex_buffer = Buffer::new(
            device_handle,
            allocator,
            resources,
            vertex_buffer_size,
            vk::BufferUsageFlags::STORAGE_BUFFER
                | vk::BufferUsageFlags::TRANSFER_DST
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            "mesh_vertices",
            MemoryLocation::GpuOnly,
        );

        let vertex_buffer_address = unsafe {
            device_handle.get_buffer_device_address(
                &vk::BufferDeviceAddressInfo::default().buffer(vertex_buffer.handle),
            )
        };

        let (mut staging_buffer, mut staging_allocation) = Buffer::create(
            device_handle,
            allocator,
            index_buffer_size + vertex_buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            "mesh_staging",
            MemoryLocation::CpuToGpu,
        );

        let vertices_record = staging_buffer.upload(&vertices, &mut staging_allocation, 0);
        staging_buffer.upload(
            indices,
            &mut staging_allocation,
            vertices_record.copy_end_offset_padded,
        );

        immediate_manager.submit(device_handle, &|command_buffer: vk::CommandBuffer| unsafe {
            device_handle.cmd_copy_buffer(
                command_buffer,
                staging_buffer.handle,
                vertex_buffer.handle,
                &[vk::BufferCopy::default()
                    .src_offset(0)
                    .dst_offset(0)
                    .size(vertex_buffer_size)],
            );

            device_handle.cmd_copy_buffer(
                command_buffer,
                staging_buffer.handle,
                index_buffer.handle,
                &[vk::BufferCopy::default()
                    .src_offset(vertices_record.copy_end_offset_padded as u64)
                    .dst_offset(0)
                    .size(index_buffer_size)],
            );
        });

        allocator
            .free(staging_allocation)
            .expect("koi::ren::vk::mesh - failed to Free Staging Buffer allocation");

        Self {
            index_buffer,
            vertex_buffer,
            vertex_buffer_address,
        }
    }
}
