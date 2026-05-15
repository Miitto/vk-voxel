use crate::core::VkCore;
use ash::vk;
use std::mem::ManuallyDrop;
use tracing::{debug, warn};

mod instance;

pub use instance::AppInfo;

#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    #[error("Failed to create Vulkan instance: {0}.")]
    InstanceCreationError(#[from] instance::InstanceCreationError),
}

impl VkCore {
    pub fn new(app_info: AppInfo) -> Result<VkCore, SetupError> {
        let instance = instance::createInstance(app_info)?;

        debug!("Created Vulkan Core.");
        Ok(VkCore {
            instance: ManuallyDrop::new(instance),
        })
    }
}
