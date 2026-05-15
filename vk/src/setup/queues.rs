use ash::vk;

use crate::core::{Queue, Queues};

#[derive(Debug, thiserror::Error)]
pub enum QueueFamilyError {
    #[error("Failed to get queue family properties: {0}")]
    FailedToGetQueueFamilyProperties(#[source] vk::Result),
    #[error("No queue family supports graphics operations")]
    NoGraphicsQueueFamily,
    #[error("No queue family supports presentation to the given surface")]
    NoPresentQueueFamily,
}

#[derive(Debug)]
pub struct QueueFamilies {
    pub graphics: u32,
    pub present: u32,
    pub compute: u32,
    pub transfer: u32,
}

pub fn get_queue_families(
    instance: &ash::Instance,
    phys_device: vk::PhysicalDevice,
) -> Result<QueueFamilies, QueueFamilyError> {
    let props = unsafe { instance.get_physical_device_queue_family_properties(phys_device) };

    let mut graphics = None;
    let mut present = None;
    let mut compute = None;
    let mut transfer = None;

    let mut found_dedicated_compute = false;
    let mut found_dedicated_transfer = false;
    let mut found_combined_graphics_present = false;

    for (index, fam) in props.iter().enumerate() {
        let index = index as u32;

        if fam.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics.is_none() {
            graphics = Some(index);
        }
        if fam.queue_flags.contains(vk::QueueFlags::COMPUTE)
            && (compute.is_none()
                || (!found_dedicated_compute
                    && !fam.queue_flags.contains(vk::QueueFlags::GRAPHICS)))
        {
            compute = Some(index);

            if !fam.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                found_dedicated_compute = true;
            }
        }
        if fam.queue_flags.contains(vk::QueueFlags::TRANSFER)
            && (transfer.is_none()
                || (!found_dedicated_transfer
                    && !fam.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && !fam.queue_flags.contains(vk::QueueFlags::COMPUTE)))
        {
            transfer = Some(index);

            if !fam.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && !fam.queue_flags.contains(vk::QueueFlags::COMPUTE)
            {
                found_dedicated_transfer = true;
            }
        }
    }

    if graphics.is_none() {
        return Err(QueueFamilyError::NoGraphicsQueueFamily);
    }
    if present.is_none() {
        return Err(QueueFamilyError::NoPresentQueueFamily);
    }
    if compute.is_none() {
        compute = graphics;
    }
    if transfer.is_none() {
        transfer = compute;
    }

    Ok(QueueFamilies {
        graphics: graphics.unwrap(),
        present: present.unwrap(),
        compute: compute.unwrap(),
        transfer: transfer.unwrap(),
    })
}

pub fn get_queues(device: &ash::Device, fams: QueueFamilies) -> Queues {
    let unique_fams = {
        let mut set = std::collections::HashSet::new();
        set.insert(fams.graphics);
        set.insert(fams.present);
        set.insert(fams.compute);
        set.insert(fams.transfer);
        set
    };

    let unique_queues = unique_fams
        .into_iter()
        .map(|family| Queue {
            family,
            queue: unsafe { device.get_device_queue(family, 0) },
        })
        .collect::<Vec<_>>();

    Queues {
        graphics: *unique_queues
            .iter()
            .find(|q| q.family == fams.graphics)
            .expect("No graphics queue returned from device"),
        present: *unique_queues
            .iter()
            .find(|q| q.family == fams.present)
            .expect("No present queue returned from device"),
        compute: *unique_queues
            .iter()
            .find(|q| q.family == fams.compute)
            .expect("No compute queue returned from device"),
        transfer: *unique_queues
            .iter()
            .find(|q| q.family == fams.transfer)
            .expect("No transfer queue returned from device"),
    }
}
