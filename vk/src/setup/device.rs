use ash::vk;
use tracing::debug;

use crate::core::DeviceInfo;

#[derive(Debug, thiserror::Error)]
pub enum DeviceSelectError {
    #[error("Failed to enumerate physical devices: {0}")]
    FailedToEnumerateDevices(#[source] vk::Result),
    #[error("No suitable physical devices found")]
    NoSuitableDevices,
}

pub struct DeviceSelectInfo<'a, F>
where
    F: Fn(&crate::core::DeviceInfo) -> usize,
{
    /// Extensions that must be supported by the device. Devices missing any of these will be
    /// excluded before the score function is run.
    pub required_extensions: &'a [String],
    /// Score function. Return 0 to exclude device, otherwise return score is used for sorting (higher
    /// is better).
    pub score_fn: F,
}

#[derive(Debug, thiserror::Error)]
pub enum DeviceCreationError {
    #[error("Failed to create logical device: {0}")]
    FailedToCreateLogicalDevice(#[from] vk::Result),
}

impl crate::core::Device {
    pub(crate) fn select<F>(
        instance: &ash::Instance,
        dev_info: DeviceSelectInfo<F>,
    ) -> Result<Vec<crate::core::DeviceInfo>, DeviceSelectError>
    where
        F: Fn(&crate::core::DeviceInfo) -> usize,
    {
        debug!("");
        debug!("Available Vulkan Physical Devices:");
        let mut phys_devices = unsafe { instance.enumerate_physical_devices() }
            .map_err(DeviceSelectError::FailedToEnumerateDevices)?
            .into_iter()
            .filter_map(|dev| {
                let exts = unsafe { instance.enumerate_device_extension_properties(dev) }
                    .ok()?
                    .iter()
                    .filter_map(|ext| {
                        unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) }
                            .to_str()
                            .ok()
                            .map(|s| s.to_string())
                    })
                    .collect::<Vec<_>>();

                let features = unsafe { instance.get_physical_device_features(dev) };

                let props = unsafe { instance.get_physical_device_properties(dev) };

                let name = unsafe { std::ffi::CStr::from_ptr(props.device_name.as_ptr()) }
                    .to_str()
                    .unwrap_or("Unknown Device")
                    .to_string();

                debug!("- {}", name);

                Some(crate::core::DeviceInfo {
                    name,
                    dev,
                    exts,
                    features,
                    props,
                })
            })
            .filter(|d| {
                dev_info
                    .required_extensions
                    .iter()
                    .all(|req| d.exts.contains(req))
            })
            .filter_map(|mut d| {
                let score = (dev_info.score_fn)(&mut d);
                if score > 0 { Some((d, score)) } else { None }
            })
            .filter(|(_, score)| *score > 0)
            .collect::<Vec<_>>();

        debug!("");
        debug!("Required Device Extensions:");
        for ext in dev_info.required_extensions {
            debug!("- {}", ext);
        }

        if phys_devices.is_empty() {
            return Err(DeviceSelectError::NoSuitableDevices);
        }

        phys_devices.sort_unstable_by_key(|(_, score)| std::cmp::Reverse(*score));

        Ok(phys_devices.into_iter().map(|(d, _)| d).collect())
    }

    pub(crate) fn new(
        instance: &ash::Instance,
        phys_device: DeviceInfo,
        queue_fams: &crate::setup::queues::QueueFamilies,
        required_extensions: &[String],
    ) -> Result<crate::core::Device, DeviceCreationError> {
        let queue_priorities = [1.0_f32];

        let required_ext_cstrs: Vec<std::ffi::CString> = required_extensions
            .iter()
            .map(|s| std::ffi::CString::new(s.as_str()).expect("Invalid Extension Name"))
            .collect();
        let required_ext_ptrs: Vec<*const i8> =
            required_ext_cstrs.iter().map(|s| s.as_ptr()).collect();

        let unique_queue_fams = {
            let mut set = std::collections::HashSet::new();
            set.insert(queue_fams.graphics);
            set.insert(queue_fams.present);
            set.insert(queue_fams.compute);
            set.insert(queue_fams.transfer);
            set
        };

        let queue_infos: Vec<vk::DeviceQueueCreateInfo> = unique_queue_fams
            .into_iter()
            .map(|fam| vk::DeviceQueueCreateInfo {
                queue_family_index: fam,
                p_queue_priorities: queue_priorities.as_ptr(),
                queue_count: queue_priorities.len() as u32,
                ..Default::default()
            })
            .collect();

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&required_ext_ptrs);

        let logical =
            unsafe { instance.create_device(phys_device.dev, &device_create_info, None) }?;

        Ok(crate::core::Device {
            physical: phys_device,
            logical,
        })
    }
}
