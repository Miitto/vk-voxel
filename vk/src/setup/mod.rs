use crate::core::{Device, Instance, Queues, Surface, Swapchain, VkCore};
use ash::vk;
use std::mem::ManuallyDrop;
use tracing::debug;

mod device;
mod frame;
mod instance;
mod queues;
mod swapchain;

pub use instance::AppInfo;

pub use swapchain::SwapchainCreationError;

#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    #[error("Failed to create Vulkan instance: {0}")]
    InstanceCreation(#[from] instance::InstanceCreationError),
    #[error("Failed to create Vulkan surface: {0}")]
    SurfaceCreation(#[from] instance::SurfaceError),
    #[error("Failed to select Vulkan physical devices: {0}")]
    DeviceScore(#[from] device::DeviceSelectError),
    #[error("Failed to get Vulkan queue families: {0}")]
    QueueFamily(#[from] queues::QueueFamilyError),
    #[error("Failed to create Vulkan logical device: {0}")]
    LogicalDeviceCreation(#[from] device::DeviceCreationError),
    #[error("Failed to create Vulkan memory allocator: {0}")]
    AllocatorCreation(#[source] vk::Result),
    #[error("Failed to create Vulkan swapchain: {0}")]
    SwapchainCreation(#[from] swapchain::SwapchainCreationError),
    #[error("Failed to create frame resources: {0}")]
    FrameResourceCreation(#[from] frame::FrameResourceCreationError),
}

impl VkCore {
    pub fn new(app_info: AppInfo, window: &winit::window::Window) -> Result<VkCore, SetupError> {
        let instance = Instance::new(app_info)?;

        let surface = Surface::new(&instance, window)?;

        let required_device_exts = vec![
            ash::khr::swapchain::NAME
                .to_str()
                .expect("Invalid Ext Name")
                .to_string(),
        ];

        let phys_devices = Device::select(
            &instance,
            device::DeviceSelectInfo {
                required_extensions: &required_device_exts,
                score_fn: |info| {
                    let mut score = 0;

                    if info.props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                        score += 1000;
                    }

                    score
                },
            },
        )?;
        debug!("");
        debug!("Suitable Physical Devices:");
        phys_devices.iter().for_each(|d| debug!("-  {}", d.name));

        let phys_device = phys_devices
            .into_iter()
            .next()
            .expect("No phsical devices without error?");

        let queue_fams = Queues::find_families(&instance, &surface, phys_device.dev)?;
        debug!("");
        debug!("Selected Physical Device Queue Families:");
        debug!("- Graphics: {}", queue_fams.graphics);
        debug!("- Present: {}", queue_fams.present);
        debug!("- Compute: {}", queue_fams.compute);
        debug!("- Transfer: {}", queue_fams.transfer);

        let mut features13 = vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);
        let mut features12 = vk::PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(true)
            .descriptor_indexing(true)
            .descriptor_binding_partially_bound(true)
            .descriptor_binding_variable_descriptor_count(true)
            .runtime_descriptor_array(true)
            .descriptor_binding_sampled_image_update_after_bind(true)
            .descriptor_binding_storage_image_update_after_bind(true)
            .descriptor_binding_uniform_buffer_update_after_bind(true)
            .descriptor_binding_storage_buffer_update_after_bind(true);
        let mut features11 = vk::PhysicalDeviceVulkan11Features::default();
        let features = vk::PhysicalDeviceFeatures2::default()
            .features(vk::PhysicalDeviceFeatures::default().sampler_anisotropy(true))
            .push_next(&mut features11)
            .push_next(&mut features12)
            .push_next(&mut features13);
        let device = Device::new(
            &instance,
            phys_device,
            &queue_fams,
            features,
            &required_device_exts,
        )?;

        let queues = Queues::retrieve(&device, queue_fams);

        let allocator = {
            let mut create_info =
                vk_mem::AllocatorCreateInfo::new(&instance, &device, device.physical.dev);
            create_info.flags = vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS;
            create_info.vulkan_api_version = vk::make_api_version(0, 1, 3, 0);
            unsafe { vk_mem::Allocator::new(create_info) }
        }
        .map_err(SetupError::AllocatorCreation)?;

        let swapchain = Swapchain::new(&instance, &surface, &device)?;

        let mut error = None;
        let frame = {
            let frame_res: [Result<crate::core::Frame, frame::FrameResourceCreationError>;
                crate::FRAMES_IN_FLIGHT] = std::array::from_fn(|_| {
                let f = crate::core::Frame::new(&device, &queues);
                if let Err(e) = &f {
                    error = Some(*e);
                }
                f
            });
            if let Some(e) = error {
                return Err(e.into());
            }
            frame_res.map(|f| f.expect("Frame resource creation failed with error"))
        };

        let transfer_pool = {
            let pool_info = vk::CommandPoolCreateInfo {
                queue_family_index: queues.transfer.family,
                flags: vk::CommandPoolCreateFlags::TRANSIENT,
                ..Default::default()
            };

            let pool = unsafe { device.logical.create_command_pool(&pool_info, None) }
                .map_err(frame::FrameResourceCreationError::CommandPoolCreation)?;

            crate::core::CommandPool {
                pool,
                queue: queues.transfer,
            }
        };

        debug!("Created Vulkan Core.");
        Ok(VkCore {
            instance: ManuallyDrop::new(instance),
            surface: ManuallyDrop::new(surface),
            device: ManuallyDrop::new(device),
            queues,
            allocator: ManuallyDrop::new(allocator),
            swapchain,
            frame,
            transfer_pool,
        })
    }
}
