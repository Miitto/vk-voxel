use crate::core::{Device, Frame, Queues};
use ash::vk;

#[derive(Debug, thiserror::Error, Clone, Copy)]
pub enum FrameResourceCreationError {
    #[error("Failed to create command pool: {0}")]
    CommandPoolCreation(#[source] vk::Result),
    #[error("Failed to create in flight fence: {0}")]
    FenceCreation(#[source] vk::Result),
    #[error("Failed to create image available semaphore: {0}")]
    SemaphoreCreateInfo(#[source] vk::Result),
}

impl Frame {
    pub(crate) fn new(
        device: &Device,
        queues: &Queues,
    ) -> Result<Frame, FrameResourceCreationError> {
        let pools = {
            let pool_info = vk::CommandPoolCreateInfo {
                queue_family_index: queues.graphics.family,
                flags: vk::CommandPoolCreateFlags::TRANSIENT,
                ..Default::default()
            };

            let graphics = crate::core::CommandPool {
                pool: unsafe { device.create_command_pool(&pool_info, None) }
                    .map_err(FrameResourceCreationError::CommandPoolCreation)?,
                queue: queues.graphics,
            };

            let compute = {
                let compute_pool_info = vk::CommandPoolCreateInfo {
                    queue_family_index: queues.compute.family,
                    flags: vk::CommandPoolCreateFlags::TRANSIENT,
                    ..Default::default()
                };

                crate::core::CommandPool {
                    pool: unsafe { device.create_command_pool(&compute_pool_info, None) }
                        .map_err(FrameResourceCreationError::CommandPoolCreation)?,
                    queue: queues.compute,
                }
            };

            crate::core::Pools { graphics, compute }
        };

        let in_flight_fence = {
            let create_info = vk::FenceCreateInfo {
                flags: vk::FenceCreateFlags::SIGNALED,
                ..Default::default()
            };
            unsafe { device.create_fence(&create_info, None) }
                .map_err(FrameResourceCreationError::FenceCreation)?
        };

        let image_available_semaphore = {
            let create_info = vk::SemaphoreCreateInfo::default();
            unsafe { device.create_semaphore(&create_info, None) }
                .map_err(FrameResourceCreationError::SemaphoreCreateInfo)?
        };

        Ok(Frame {
            pools,
            in_flight_fence,
            image_available_semaphore,
        })
    }
}
