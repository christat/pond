use ash::{Device as DeviceHandle, vk};

pub struct DescriptorSetLayoutBuilder<'a> {
    pub bindings: Vec<vk::DescriptorSetLayoutBinding<'a>>,
}

#[allow(unused)]
impl<'a> DescriptorSetLayoutBuilder<'a> {
    pub fn default() -> Self {
        Self { bindings: vec![] }
    }

    pub fn add_binding(mut self, binding: u32, descriptor_type: vk::DescriptorType) -> Self {
        self.bindings.push(
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_count(1)
                .descriptor_type(descriptor_type),
        );
        self
    }

    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    pub fn build<T: vk::ExtendsDescriptorSetLayoutCreateInfo + ?Sized>(
        &mut self,
        device_handle: &DeviceHandle,
        shader_stages: vk::ShaderStageFlags,
        flags: Option<vk::DescriptorSetLayoutCreateFlags>,
        next: Option<&'a mut T>,
    ) -> vk::DescriptorSetLayout {
        self.bindings
            .iter_mut()
            .for_each(|binding| binding.stage_flags = binding.stage_flags | shader_stages);

        let mut create_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&self.bindings)
            .flags(flags.unwrap_or_default());

        if next.is_some() {
            create_info = create_info.push_next(next.unwrap());
        }

        unsafe {
            device_handle
                .create_descriptor_set_layout(&create_info, None)
                .expect("koi::ren::vk::descriptor - failed to Create Descriptor Set Layout")
        }
    }
}

pub struct DescriptorSetPoolSizeRatio {
    pub ty: vk::DescriptorType,
    pub ratio: f32,
}

impl DescriptorSetPoolSizeRatio {
    pub fn new(ty: vk::DescriptorType, ratio: f32) -> Self {
        Self { ty, ratio }
    }
}

pub struct DescriptorSetAllocator {
    pool: vk::DescriptorPool,
}

#[allow(unused)]
impl DescriptorSetAllocator {
    pub fn new(
        device_handle: &DeviceHandle,
        max_sets: u32,
        pool_ratios: &[DescriptorSetPoolSizeRatio],
    ) -> Self {
        let pool_sizes: Vec<_> = pool_ratios
            .iter()
            .map(|pool_ratio| {
                vk::DescriptorPoolSize::default()
                    .ty(pool_ratio.ty)
                    .descriptor_count((pool_ratio.ratio * max_sets as f32) as u32)
            })
            .collect();

        let create_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(max_sets)
            .pool_sizes(&pool_sizes);

        let pool = unsafe {
            device_handle
                .create_descriptor_pool(&create_info, None)
                .expect("koi::ren::vk::descriptor - failed to Create Descriptor Pool")
        };

        Self { pool }
    }

    pub fn allocate(
        &mut self,
        device_handle: &DeviceHandle,
        layouts: &[vk::DescriptorSetLayout],
    ) -> vk::DescriptorSet {
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.pool)
            .set_layouts(layouts);

        unsafe {
            device_handle
                .allocate_descriptor_sets(&allocate_info)
                .expect("koi::ren::vk::descriptor - failed to Allocate Descriptor Set")[0]
        }
    }

    pub fn reset_pool(&mut self, device_handle: &DeviceHandle) {
        unsafe {
            device_handle
                .reset_descriptor_pool(self.pool, vk::DescriptorPoolResetFlags::empty())
                .expect("koi::ren::vk::descriptor - failed to Reset Descriptor Pool")
        };
    }

    pub fn drop(&mut self, device_handle: &DeviceHandle) {
        unsafe { device_handle.destroy_descriptor_pool(self.pool, None) };
    }
}
