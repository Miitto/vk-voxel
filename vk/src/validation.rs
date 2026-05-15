use crate::core::Instance;
use ash::vk;
use tracing::{debug, error, trace, warn};

impl Instance {
    pub fn make_debug_messenger(&mut self) -> bool {
        let debug_utils = ash::ext::debug_utils::Instance::new(&self.entry, &self.instance);
        let debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            pfn_user_callback: Some(vulkan_debug_callback),
            ..Default::default()
        };

        if let Ok(m) = unsafe { debug_utils.create_debug_utils_messenger(&debug_create_info, None) }
        {
            self.debug_messenger = Some(m);
            true
        } else {
            false
        }
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = unsafe { std::ffi::CStr::from_ptr((*p_callback_data).p_message) };

    if _message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
        error!("Vulkan Validation Error: {}", message.to_string_lossy());
    } else if _message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING) {
        warn!("Vulkan Validation Warning: {}", message.to_string_lossy());
    } else if _message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE) {
        trace!("Vulkan Validation Info: {}", message.to_string_lossy());
    } else {
        debug!("Vulkan Validation Message: {}", message.to_string_lossy());
    }

    vk::FALSE
}
