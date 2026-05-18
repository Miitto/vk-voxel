use crate::core::{Device, Instance, Queues, Surface, Swapchain, VkCore};
use ash::vk;
use std::mem::ManuallyDrop;
use tracing::debug;

mod device;
mod instance;
mod queues;
mod swapchain;

pub use instance::AppInfo;

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
    #[error("Failed to create Vulkan swapchain: {0}")]
    SwapchainCreation(#[from] swapchain::SwapchainCreationError),
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

        let device = Device::new(&instance, phys_device, &queue_fams, &required_device_exts)?;

        let queues = Queues::retrieve(&device, queue_fams);

        let swapchain = Swapchain::new(&instance, &surface, &device)?;

        debug!("Created Vulkan Core.");
        Ok(VkCore {
            instance: ManuallyDrop::new(instance),
            surface: ManuallyDrop::new(surface),
            device: ManuallyDrop::new(device),
            queues,
            swapchain: ManuallyDrop::new(swapchain),
        })
    }
}
