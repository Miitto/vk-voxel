use crate::core::VkCore;
use ash::vk;
use std::mem::ManuallyDrop;
use tracing::debug;

mod device;
mod instance;
mod queues;

pub use instance::AppInfo;

#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    #[error("Failed to create Vulkan instance: {0}")]
    InstanceCreation(#[from] instance::InstanceCreationError),
    #[error("Failed to select Vulkan physical devices: {0}")]
    DeviceScore(#[from] device::DeviceSelectError),
    #[error("Failed to get Vulkan queue families: {0}")]
    QueueFamily(#[from] queues::QueueFamilyError),
    #[error("Failed to create Vulkan logical device: {0}")]
    LogicalDeviceCreation(#[from] device::DeviceCreationError),
}

impl VkCore {
    pub fn new(app_info: AppInfo) -> Result<VkCore, SetupError> {
        let instance = instance::create_instance(app_info)?;

        let phys_devices = device::select_device(
            &instance.instance,
            device::DeviceSelectInfo {
                required_extensions: vec!["VK_KHR_swapchain".to_string()],
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

        let queue_fams = queues::get_queue_families(&instance.instance, phys_device.dev)?;
        debug!("");
        debug!("Selected Physical Device Queue Families:");
        debug!("- Graphics: {}", queue_fams.graphics);
        debug!("- Present: {}", queue_fams.present);
        debug!("- Compute: {}", queue_fams.compute);
        debug!("- Transfer: {}", queue_fams.transfer);

        let device = device::create_device(&instance.instance, phys_device, &queue_fams)?;

        let queues = queues::get_queues(&device.logical, queue_fams);

        debug!("Created Vulkan Core.");
        Ok(VkCore {
            instance: ManuallyDrop::new(instance),
            device: ManuallyDrop::new(device),
            queues,
        })
    }
}
