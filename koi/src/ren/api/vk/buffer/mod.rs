use super::resource_allocator::AllocatedResources;

use ash::{Device as DeviceHandle, vk};
use gpu_allocator::{MemoryLocation, vulkan as vka};

#[allow(unused)]
pub struct Buffer {
    pub handle: vk::Buffer,
    pub size: vk::DeviceSize,
    pub memory: vk::DeviceMemory,

    pub usage: vk::BufferUsageFlags,
    pub location: MemoryLocation,
    pub min_alignment: usize,
}

impl Buffer {
    pub fn create(
        device_handle: &DeviceHandle,
        allocator: &mut vka::Allocator,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        name: &str,
        location: MemoryLocation,
    ) -> (Self, vka::Allocation) {
        let create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device_handle
                .create_buffer(&create_info, None)
                .expect("koi::ren::vk::buffer - failed to Create Buffer")
        };

        let requirements = unsafe { device_handle.get_buffer_memory_requirements(buffer) };
        let allocation = allocator
            .allocate(&vka::AllocationCreateDesc {
                name,
                requirements,
                location,
                linear: true,
                allocation_scheme: vka::AllocationScheme::DedicatedBuffer(buffer),
            })
            .expect("koi::ren::vk::buffer - failed to Allocate Buffer");

        let memory = unsafe { allocation.memory() };
        unsafe {
            device_handle
                .bind_buffer_memory(buffer, memory, 0)
                .expect("koi::ren::vk::buffer - failed to Bind Buffer")
        }

        (
            Self {
                handle: buffer,
                size,
                memory,
                usage,
                location,
                min_alignment: requirements.alignment as usize,
            },
            allocation,
        )
    }

    pub fn new(
        device_handle: &DeviceHandle,
        allocator: &mut vka::Allocator,
        resources: &mut AllocatedResources,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        name: &str,
        location: MemoryLocation,
    ) -> Self {
        let (buffer, allocation) =
            Self::create(device_handle, allocator, size, usage, name, location);
        resources.add_buffer(buffer.handle, allocation);
        buffer
    }

    pub fn resize(
        &mut self,
        device_handle: &DeviceHandle,
        allocator: &mut vka::Allocator,
        allocation: vka::Allocation,
        size: vk::DeviceSize,
        name: &str,
    ) -> vka::Allocation {
        unsafe { device_handle.destroy_buffer(self.handle, None) };
        allocator
            .free(allocation)
            .expect("ren::vk::buffer - failed to Free Resize Buffer");

        let (new_buffer, new_allocation) = Self::create(
            device_handle,
            allocator,
            size,
            self.usage,
            name,
            self.location,
        );
        self.handle = new_buffer.handle;
        self.size = size;
        self.memory = new_buffer.memory;

        new_allocation
    }

    pub fn upload<T: Copy>(
        &mut self,
        src: &[T],
        dst: &mut vka::Allocation,
        start_offset: usize,
    ) -> presser::CopyRecord {
        presser::copy_from_slice_to_offset_with_align(src, dst, start_offset, self.min_alignment)
            .expect("koi::ren::vk::buffer - failed to Upload to Buffer")
    }
}
