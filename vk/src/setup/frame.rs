use crate::core::{Device, Frame, Queues};
use ash::vk;

#[derive(Debug, thiserror::Error, Clone, Copy)]
pub enum FrameResourceCreationError {
    #[error("Failed to create command pool: {0}")]
    CommandPoolCreation(#[source] vk::Result),
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

            let compute = if queues.graphics.family != queues.compute.family {
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
            } else {
                graphics
            };

            crate::core::Pools { graphics, compute }
        };

        Ok(Frame { pools })
    }
}
