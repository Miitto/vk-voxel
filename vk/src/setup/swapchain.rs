use crate::core::{Surface, Swapchain};
use ash::vk;

#[derive(Debug, thiserror::Error)]
pub enum SwapchainCreationError {
    #[error("Failed to query surface capabilities: {0}")]
    SurfaceCapabilities(#[source] vk::Result),
    #[error("Failed to query surface formats: {0}")]
    SurfaceFormats(#[source] vk::Result),
    #[error("Failed to query surface present modes: {0}")]
    SurfacePresentModes(#[source] vk::Result),
    #[error("Failed to create swapchain: {0}")]
    Creation(#[source] vk::Result),
    #[error("Failed to get swapchain images: {0}")]
    ImageRetrieval(#[source] vk::Result),
    #[error("Failed to create image view for swapchain image: {0}")]
    ViewCreation(#[source] vk::Result),
    #[error("Failed to create semaphore for swapchain synchronization: {0}")]
    SemaphoreCreation(#[source] vk::Result),
}

impl Swapchain {
    pub(crate) fn new(
        instance: &ash::Instance,
        surface: &Surface,
        device: &crate::core::Device,
    ) -> Result<Swapchain, SwapchainCreationError> {
        let caps = unsafe {
            surface.get_physical_device_surface_capabilities(device.physical.dev, surface.surface)
        }
        .map_err(SwapchainCreationError::SurfaceCapabilities)?;

        let formats = unsafe {
            surface.get_physical_device_surface_formats(device.physical.dev, surface.surface)
        }
        .map_err(SwapchainCreationError::SurfaceFormats)?;

        let present_modes = unsafe {
            surface.get_physical_device_surface_present_modes(device.physical.dev, surface.surface)
        }
        .map_err(SwapchainCreationError::SurfacePresentModes)?;

        let surface_format = formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&formats[0]);

        let present_mode = present_modes
            .iter()
            .find(|&m| *m == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(&vk::PresentModeKHR::FIFO);

        let create_info = vk::SwapchainCreateInfoKHR {
            surface: surface.surface,
            min_image_count: 3.max(caps.min_image_count).min(caps.max_image_count),
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: caps.current_extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform: caps.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: *present_mode,
            ..Default::default()
        };

        let loader = ash::khr::swapchain::Device::new(&instance, &device);
        let swapchain = unsafe { loader.create_swapchain(&create_info, None) }
            .map_err(SwapchainCreationError::Creation)?;

        let images = unsafe { loader.get_swapchain_images(swapchain) }
            .map_err(SwapchainCreationError::ImageRetrieval)?;
        let mut views = Vec::with_capacity(images.len());

        let mut view_create = vk::ImageViewCreateInfo::default()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(surface_format.format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let mut sems = Vec::with_capacity(images.len());
        let mut sem_create_info = vk::SemaphoreCreateInfo::default();

        for image in &images {
            view_create.image = *image;

            let view = unsafe { device.logical.create_image_view(&view_create, None) }
                .map_err(SwapchainCreationError::ViewCreation)?;
            views.push(view);

            let sem = unsafe { device.logical.create_semaphore(&sem_create_info, None) }
                .map_err(SwapchainCreationError::SemaphoreCreation)?;
            sems.push(sem);
        }

        Ok(Swapchain {
            swapchain,
            images,
            views,
            sems,
            loader,
            info: crate::core::SwapchainInfo {
                format: surface_format.format,
                extent: caps.current_extent,
                present_mode: *present_mode,
            },
        })
    }
}
