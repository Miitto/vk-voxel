use std::mem::ManuallyDrop;
use tracing::debug;

use ash::vk;

pub struct VkCore {
    pub instance: ManuallyDrop<Instance>,
    pub device: ManuallyDrop<Device>,
    pub queues: Queues,
}

pub struct Instance {
    pub entry: ManuallyDrop<ash::Entry>,
    pub instance: ManuallyDrop<ash::Instance>,
    #[cfg(debug_assertions)]
    pub debug_messenger: Option<ash::vk::DebugUtilsMessengerEXT>,
}

pub struct DeviceInfo {
    pub name: String,
    pub dev: vk::PhysicalDevice,
    pub exts: Vec<String>,
    pub features: vk::PhysicalDeviceFeatures,
    pub props: vk::PhysicalDeviceProperties,
}

pub struct Device {
    pub physical: DeviceInfo,
    pub logical: ash::Device,
}

#[derive(Debug, Clone, Copy)]
pub struct Queue {
    pub family: u32,
    pub queue: vk::Queue,
}

pub struct Queues {
    pub graphics: Queue,
    pub present: Queue,
    pub compute: Queue,
    pub transfer: Queue,
}

impl Drop for VkCore {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.device);
            ManuallyDrop::drop(&mut self.instance);
        }
        debug!("Vulkan Core dropped.");
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            #[cfg(debug_assertions)]
            {
                let debug_utils = ash::ext::debug_utils::Instance::new(&self.entry, &self.instance);
                debug_utils.destroy_debug_utils_messenger(self.debug_messenger.unwrap(), None);
            }
            self.instance.destroy_instance(None);
            ManuallyDrop::drop(&mut self.instance);
            ManuallyDrop::drop(&mut self.entry);
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.logical.destroy_device(None);
        }
    }
}
