use super::{device::Device, instance::Instance, surface::Surface};
use crate::ren::settings::Resolution;

use ash::{Device as DeviceHandle, khr, vk};
use std::cmp;

pub struct SurfaceSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SurfaceSupport {
    pub fn new(
        capabilities: vk::SurfaceCapabilitiesKHR,
        formats: Vec<vk::SurfaceFormatKHR>,
        present_modes: Vec<vk::PresentModeKHR>,
    ) -> Self {
        Self {
            capabilities,
            formats,
            present_modes,
        }
    }
}

#[derive(Debug)]

pub enum SwapchainError {
    NoSurfaceFormats,
    NoPresentModes,
}

#[allow(unused)]
pub struct Swapchain {
    pub device: khr::swapchain::Device,
    pub khr: vk::SwapchainKHR,
    pub format: vk::Format,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
}

impl Swapchain {
    pub fn new(
        instance: &Instance,
        device: &Device,
        surface: &Surface,
        resolution: &Resolution,
    ) -> Result<(Swapchain, SurfaceSupport), SwapchainError> {
        let surface_support = query_surface_support(device.physical_device, &surface)?;
        let swapchain = Self::create(instance, device, surface, &surface_support, resolution);
        Ok((swapchain, surface_support))
    }

    pub fn create(
        instance: &Instance,
        device: &Device,
        surface: &Surface,
        surface_support: &SurfaceSupport,
        resolution: &Resolution,
    ) -> Self {
        let surface_format = select_surface_format(
            &surface_support,
            vk::Format::B8G8R8A8_UNORM,
            vk::ColorSpaceKHR::SRGB_NONLINEAR,
        );
        let present_mode = select_present_mode(&surface_support, vk::PresentModeKHR::FIFO);
        let swapchain_extent = select_swapchain_extent(&surface_support, resolution);
        let min_image_count = select_swapchain_min_image_count(&surface_support);
        let (image_sharing_mode, queue_family_indices) = get_queue_family_config(device);

        let mut create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.khr)
            .min_image_count(min_image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(swapchain_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .image_sharing_mode(image_sharing_mode)
            .pre_transform(surface_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        if !queue_family_indices.is_empty() {
            create_info = create_info.queue_family_indices(&queue_family_indices)
        };

        let swapchain_device = khr::swapchain::Device::new(&instance.handle, &device.handle);

        let khr = unsafe {
            swapchain_device
                .create_swapchain(&create_info, None)
                .expect("koi::ren::vk::swapchain - failed to create Swapchain")
        };
        let images = unsafe {
            swapchain_device
                .get_swapchain_images(khr)
                .expect("koi::ren::vk::swapchain - failed to get swapchain Images")
        };
        let image_views: Vec<vk::ImageView> = images
            .iter()
            .map(|swapchain_image| {
                let create_info = vk::ImageViewCreateInfo::default()
                    .image(swapchain_image.clone())
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(
                        vk::ComponentMapping::default()
                            .r(vk::ComponentSwizzle::IDENTITY)
                            .g(vk::ComponentSwizzle::IDENTITY)
                            .b(vk::ComponentSwizzle::IDENTITY)
                            .a(vk::ComponentSwizzle::IDENTITY),
                    )
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    );

                unsafe {
                    device
                        .handle
                        .create_image_view(&create_info, None)
                        .expect("koi::ren::vk::swapchain - failed to get swapchain Image View")
                }
            })
            .collect();

        Self {
            device: swapchain_device,
            khr,
            format: surface_format.format,
            images,
            image_views,
            extent: swapchain_extent,
        }
    }

    pub fn drop(&mut self, device_handle: &DeviceHandle) {
        unsafe {
            self.device.destroy_swapchain(self.khr, None);
            self.images.clear(); // Swaphain owns the images; no need to destroy
            self.image_views
                .iter()
                .for_each(|image_view| device_handle.destroy_image_view(*image_view, None));
            self.image_views.clear();
        };
    }
}

fn query_surface_support(
    physical_device: vk::PhysicalDevice,
    surface: &Surface,
) -> Result<SurfaceSupport, SwapchainError> {
    let capabilities = unsafe {
        surface
            .instance
            .get_physical_device_surface_capabilities(physical_device, surface.khr)
            .expect(
                "koi::ren::vk::swapchain - failed to query physical device surface capabilities",
            )
    };

    let formats = unsafe {
        surface
            .instance
            .get_physical_device_surface_formats(physical_device, surface.khr)
            .expect("koi::ren::vk::swapchain - failed to query physical device surface formats")
    };
    if formats.is_empty() {
        return Err(SwapchainError::NoSurfaceFormats);
    }

    let present_modes = unsafe {
        surface
            .instance
            .get_physical_device_surface_present_modes(physical_device, surface.khr)
            .expect(
                "koi::ren::vk::swapchain - failed to query physical device surface present modes",
            )
    };
    if present_modes.is_empty() {
        return Err(SwapchainError::NoPresentModes);
    }

    Ok(SurfaceSupport::new(capabilities, formats, present_modes))
}

fn select_surface_format(
    surface_support: &SurfaceSupport,
    desired_format: vk::Format,
    desired_color_space: vk::ColorSpaceKHR,
) -> vk::SurfaceFormatKHR {
    match surface_support.formats.iter().find(|&format| {
        format.format == desired_format && format.color_space == desired_color_space
    }) {
        Some(format) => format.clone(),
        None => surface_support.formats.first().unwrap().clone(),
    }
}

fn select_present_mode(
    surface_support: &SurfaceSupport,
    desired_present_mode: vk::PresentModeKHR,
) -> vk::PresentModeKHR {
    match surface_support
        .present_modes
        .iter()
        .find(|&&present_mode| present_mode == desired_present_mode)
    {
        Some(present_mode) => present_mode.clone(),
        None => vk::PresentModeKHR::FIFO,
    }
}

fn select_swapchain_extent(
    surface_support: &SurfaceSupport,
    resolution: &Resolution,
) -> vk::Extent2D {
    let vk::SurfaceCapabilitiesKHR {
        min_image_extent,
        max_image_extent,
        ..
    } = surface_support.capabilities;
    vk::Extent2D::default()
        .width(cmp::min(
            cmp::max(min_image_extent.width, resolution.width),
            max_image_extent.width,
        ))
        .height(cmp::min(
            cmp::max(min_image_extent.height, resolution.height),
            max_image_extent.height,
        ))
}

fn select_swapchain_min_image_count(surface_support: &SurfaceSupport) -> u32 {
    let vk::SurfaceCapabilitiesKHR {
        min_image_count,
        max_image_count,
        ..
    } = surface_support.capabilities;
    let upper_bound = if max_image_count > 0 {
        max_image_count
    } else {
        u32::MAX
    };
    cmp::min(upper_bound, cmp::max(min_image_count, min_image_count + 1))
}

fn get_queue_family_config(device: &Device) -> (vk::SharingMode, Vec<u32>) {
    let Device {
        queue_families: qf, ..
    } = device;
    match qf.graphics_family_index == qf.present_family_index {
        true => return (vk::SharingMode::EXCLUSIVE, vec![]),
        false => (vk::SharingMode::CONCURRENT, qf.get_unique_indices()),
    }
}
