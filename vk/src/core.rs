use std::mem::ManuallyDrop;
use tracing::{debug, error};

use ash::vk;

pub struct VkCore {
    pub instance: ManuallyDrop<Instance>,
    pub surface: ManuallyDrop<Surface>,
    pub device: ManuallyDrop<Device>,
    pub queues: Queues,
    pub allocator: ManuallyDrop<vk_mem::Allocator>,
    pub swapchain: Swapchain,
    pub frame: [Frame; crate::FRAMES_IN_FLIGHT],
    pub transfer_pool: CommandPool,
}

#[derive(Debug, Default)]
pub struct Frame {
    pub pools: Pools,
    pub in_flight_fence: vk::Fence,
    pub image_available_semaphore: vk::Semaphore,
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

#[derive(Debug, Clone, Copy, Default)]
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

#[derive(Debug, Clone, Copy, Default)]
pub struct SwapchainInfo {
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub present_mode: vk::PresentModeKHR,
    pub color_space: vk::ColorSpaceKHR,
}

pub struct Swapchain {
    pub swapchain: ash::vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub views: Vec<vk::ImageView>,
    /// Per swapchain image "render finished" semaphores.
    pub sems: Vec<vk::Semaphore>,
    pub info: SwapchainInfo,
    pub loader: ash::khr::swapchain::Device,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CommandPool {
    pub pool: ash::vk::CommandPool,
    pub queue: Queue,
}

#[derive(Debug, Default)]
pub struct Pools {
    pub graphics: CommandPool,
    pub compute: CommandPool,
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

mod swapchain_impls {
    use super::*;
    impl std::ops::Deref for Swapchain {
        type Target = ash::khr::swapchain::Device;

        fn deref(&self) -> &Self::Target {
            &self.loader
        }
    }

    impl Swapchain {
        /// On success, returns the next image index and whether the swapchain is suboptimal.
        pub fn next_image(
            &self,
            signal_semaphore: &vk::Semaphore,
        ) -> Result<(u32, bool), vk::Result> {
            unsafe {
                self.loader.acquire_next_image(
                    self.swapchain,
                    u64::MAX,
                    *signal_semaphore,
                    vk::Fence::null(),
                )
            }
        }
    }
}

impl Drop for VkCore {
    fn drop(&mut self) {
        unsafe {
            if let Err(e) = self.device.device_wait_idle() {
                error!("Failed to wait for device idle during VkCore drop: {e}");
            }

            self.device
                .destroy_command_pool(self.transfer_pool.pool, None);
            for frame in &mut self.frame {
                frame.destroy(&self.device);
            }

            self.swapchain.destroy(&self.device);

            ManuallyDrop::drop(&mut self.allocator);

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

impl Swapchain {
    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            for sem in &self.sems {
                device.destroy_semaphore(*sem, None);
            }
            for view in &self.views {
                device.destroy_image_view(*view, None);
            }
            self.loader.destroy_swapchain(self.swapchain, None);
        }
    }
}

impl Frame {
    pub(super) fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_command_pool(self.pools.graphics.pool, None);
            device.destroy_command_pool(self.pools.compute.pool, None);

            device.destroy_fence(self.in_flight_fence, None);
            device.destroy_semaphore(self.image_available_semaphore, None);
        }
    }
}

impl std::ops::Deref for VkCore {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.device.logical
    }
}

impl VkCore {
    pub fn aquire_next_image(
        &self,
        signal_semaphore: &vk::Semaphore,
    ) -> Result<(u32, bool), vk::Result> {
        self.swapchain.next_image(signal_semaphore)
    }

    pub fn reset_frame(&self, idx: usize) -> ash::prelude::VkResult<()> {
        unsafe {
            self.device.reset_command_pool(
                self.frame[idx].pools.graphics.pool,
                vk::CommandPoolResetFlags::empty(),
            )?;
            self.device.reset_command_pool(
                self.frame[idx].pools.compute.pool,
                vk::CommandPoolResetFlags::empty(),
            )?;
        }
        Ok(())
    }

    pub fn remake_swapchain(&mut self) -> Result<(), crate::setup::SwapchainCreationError> {
        let new_swapchain = self.swapchain.remake(&self.surface, &self.device)?;
        _ = unsafe { self.device.device_wait_idle() };
        self.swapchain.destroy(&self.device);

        self.swapchain = new_swapchain;

        Ok(())
    }
}
