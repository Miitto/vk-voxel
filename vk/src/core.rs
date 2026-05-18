use std::mem::ManuallyDrop;
use tracing::debug;

use ash::vk;

pub struct VkCore {
    pub instance: ManuallyDrop<Instance>,
    pub surface: ManuallyDrop<Surface>,
    pub device: ManuallyDrop<Device>,
    pub queues: Queues,
    pub swapchain: ManuallyDrop<Swapchain>,
}

pub struct Instance {
    pub entry: ManuallyDrop<ash::Entry>,
    pub instance: ManuallyDrop<ash::Instance>,
    #[cfg(debug_assertions)]
    pub debug_messenger: Option<ash::vk::DebugUtilsMessengerEXT>,
}

pub struct Surface {
    pub surface: vk::SurfaceKHR,
    pub loader: ash::khr::surface::Instance,
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

pub struct SwapchainInfo {
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub present_mode: vk::PresentModeKHR,
}

pub struct Swapchain {
    pub swapchain: ash::vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub views: Vec<vk::ImageView>,
    pub sems: Vec<vk::Semaphore>,
    pub info: SwapchainInfo,
    pub loader: ash::khr::swapchain::Device,
}

mod instance_surface_impls {
    use super::*;
    impl AsRef<ash::Entry> for Instance {
        fn as_ref(&self) -> &ash::Entry {
            &self.entry
        }
    }

    impl AsRef<ash::Instance> for Instance {
        fn as_ref(&self) -> &ash::Instance {
            &self.instance
        }
    }

    impl std::ops::Deref for Instance {
        type Target = ash::Instance;

        fn deref(&self) -> &Self::Target {
            &self.instance
        }
    }

    impl std::ops::Deref for Surface {
        type Target = ash::khr::surface::Instance;

        fn deref(&self) -> &Self::Target {
            &self.loader
        }
    }

    impl AsRef<vk::SurfaceKHR> for Surface {
        fn as_ref(&self) -> &vk::SurfaceKHR {
            &self.surface
        }
    }

    impl AsRef<ash::khr::surface::Instance> for Surface {
        fn as_ref(&self) -> &ash::khr::surface::Instance {
            &self.loader
        }
    }
}

mod device_queue_impls {
    use super::*;
    impl AsRef<ash::Device> for Device {
        fn as_ref(&self) -> &ash::Device {
            &self.logical
        }
    }

    impl std::ops::Deref for Device {
        type Target = ash::Device;

        fn deref(&self) -> &Self::Target {
            &self.logical
        }
    }

    impl AsRef<vk::Queue> for Queue {
        fn as_ref(&self) -> &vk::Queue {
            &self.queue
        }
    }

    impl std::ops::Deref for Queue {
        type Target = vk::Queue;

        fn deref(&self) -> &Self::Target {
            &self.queue
        }
    }
}

impl std::ops::Deref for Swapchain {
    type Target = ash::khr::swapchain::Device;

    fn deref(&self) -> &Self::Target {
        &self.loader
    }
}

impl Drop for VkCore {
    fn drop(&mut self) {
        unsafe {
            for sem in &self.swapchain.sems {
                self.device.destroy_semaphore(*sem, None);
            }
            for view in &self.swapchain.views {
                self.device.destroy_image_view(*view, None);
            }
            ManuallyDrop::drop(&mut self.swapchain);

            ManuallyDrop::drop(&mut self.surface);
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

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.surface, None);
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

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_swapchain(self.swapchain, None);
        }
    }
}
