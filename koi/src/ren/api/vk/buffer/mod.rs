use super::resource_allocator::AllocatedResources;

use ash::{vk, Device as DeviceHandle};
use gpu_allocator::{vulkan as vka, MemoryLocation};

#[allow(unused)]
pub struct Buffer {
    pub handle: vk::Buffer,
    pub size: vk::DeviceSize,
    pub memory: vk::DeviceMemory,

    pub usage: vk::BufferUsageFlags,
    pub sharing_mode: vk::SharingMode,
    pub location: MemoryLocation,
    pub linear: bool,
}

impl Buffer {
    pub fn create(
        device_handle: &DeviceHandle,
        allocator: &mut vka::Allocator,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        sharing_mode: vk::SharingMode,
        name: &str,
        location: MemoryLocation,
        linear: bool
    ) -> (Self, vka::Allocation) {
        let create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(sharing_mode);

        let buffer = unsafe { device_handle.create_buffer(&create_info, None).expect("koi::ren::vk::buffer - failed to Create Buffer") };

        let requirements = unsafe { device_handle.get_buffer_memory_requirements(buffer) };
        let allocation = allocator.allocate(&vka::AllocationCreateDesc {
            name,
            requirements,
            location,
            linear,
            allocation_scheme: vka::AllocationScheme::DedicatedBuffer(buffer)
        }).expect("koi::ren::vk::buffer - failed to Allocate Buffer");

        let memory = unsafe { allocation.memory() };
        unsafe { device_handle.bind_buffer_memory(buffer, memory, 0).expect("koi::ren::vk::buffer - failed to Bind Buffer") }

        (Self { handle: buffer, size, memory, usage, sharing_mode, location, linear }, allocation)
    }

    pub fn new(
        device_handle: &DeviceHandle,
        allocator: &mut vka::Allocator,
        resources: &mut AllocatedResources,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        sharing_mode: vk::SharingMode,
        name: &str,
        location: MemoryLocation,
        linear: bool

    ) -> Self {
        let (buffer, allocation) = Self::create(device_handle, allocator, size, usage, sharing_mode, name, location, linear);
        resources.add_buffer(buffer.handle, allocation);
        buffer
    }

    pub fn resize(&mut self, device_handle: &DeviceHandle, allocator: &mut vka::Allocator, allocation: vka::Allocation, size: vk::DeviceSize, name: &str) -> vka::Allocation {
        unsafe { device_handle.destroy_buffer(self.handle, None) };
        allocator.free(allocation).expect("ren::vk::buffer - failed to Free Resize Buffer");

        let (new_buffer, new_allocation) = Self::create(device_handle, allocator, size, self.usage, self.sharing_mode, name, self.location, self.linear);
        self.handle = new_buffer.handle;
        self.size = size;
        self.memory = new_buffer.memory;
        
        new_allocation
    }

    pub fn upload<T: Copy>(&mut self, src: &[T], dst: &mut vka::Allocation, min_alignment: usize) {
        presser::copy_from_slice_to_offset_with_align(src, dst, 0, min_alignment).expect("koi::ren::vk::buffer - failed ot Upload to Buffer");
    }
}