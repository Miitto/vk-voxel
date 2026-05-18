use crate::ToCBytes;
use crate::core::Instance;
use std::{ffi::CStr, mem::ManuallyDrop};
use tracing::{debug, error};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

#[derive(Debug, thiserror::Error)]
pub enum InstanceCreationError {
    #[error("Failed to load Vulkan entry")]
    EntryLoadError,
    #[error("Failed to enumerate Vulkan instance extensions")]
    FailedToEnumerateExtensions,
    #[error("Missing required Vulkan extensions: {0:?}")]
    MissingExtensions(Vec<String>),
    #[error("Failed to create Vulkan instance")]
    CreationFailed,
    #[error("Failed to create Vulkan debug messenger")]
    FailedToCreateDebugMessenger,
}

pub struct AppInfo {
    pub name: &'static str,
    pub version: u32,
}

#[cfg(debug_assertions)]
fn enabled_validation_layers() -> [&'static CStr; 1] {
    [c"VK_LAYER_KHRONOS_validation"]
}

#[cfg(not(debug_assertions))]
const fn enabled_validation_layers() -> [&'static CStr; 0] {
    vec![]
}

#[cfg(target_os = "windows")]
fn required_surface_extensions() -> [&'static CStr; 2] {
    [ash::khr::surface::NAME, ash::khr::win32_surface::NAME]
}

#[cfg(not(target_os = "windows"))]
compile_error!("Unsupported platform: Vulkan surface extensions not defined for this OS.");

#[cfg(debug_assertions)]
fn required_instance_extensions() -> Vec<&'static CStr> {
    let mut extensions = required_surface_extensions().to_vec();
    extensions.push(ash::ext::debug_utils::NAME);
    extensions
}

#[cfg(not(debug_assertions))]
fn required_instance_extensions() -> Vec<&'static CStr> {
    required_surface_extensions()
}

impl crate::core::Instance {
    pub(crate) fn new(info: AppInfo) -> Result<Instance, InstanceCreationError> {
        use ash::vk;

        let entry =
            unsafe { ash::Entry::load() }.map_err(|_| InstanceCreationError::EntryLoadError)?;

        debug!("");
        debug!("Available Vulkan instance extensions:");
        let available_extensions: Vec<String> =
            unsafe { entry.enumerate_instance_extension_properties(None) }
                .map_err(|_| InstanceCreationError::FailedToEnumerateExtensions)?
                .iter()
                .filter_map(|ext| ext.extension_name_as_c_str().ok())
                .map(|c| {
                    let string = c.to_string_lossy().to_string();
                    debug!("- {}", string);
                    string
                })
                .collect();

        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 4, 0),
            application_version: info.version,
            p_application_name: info.name.as_ptr() as *const i8,
            ..Default::default()
        };

        let layers = enabled_validation_layers();
        let extensions = required_instance_extensions();
        let c_layers = layers.to_cbytes();
        let c_extensions = extensions.to_cbytes();

        debug!("");
        debug!("Required Vulkan instance extensions:");
        for ext in &extensions {
            debug!("- {}", ext.to_string_lossy());
        }

        let mut missing_extensions = Vec::new();
        for ext in &extensions {
            if !available_extensions.contains(&ext.to_string_lossy().to_string()) {
                let s = ext.to_string_lossy().to_string();
                missing_extensions.push(s);
            }
        }

        if !missing_extensions.is_empty() {
            error!("Missing required Vulkan instance extensions:");
            for ext in &missing_extensions {
                error!("- {}", ext);
            }
            return Err(InstanceCreationError::MissingExtensions(missing_extensions));
        }

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            ..Default::default()
        }
        .enabled_layer_names(c_layers.as_slice())
        .enabled_extension_names(c_extensions.as_slice());

        let instance = unsafe { entry.create_instance(&create_info, None) }
            .map_err(|_| InstanceCreationError::CreationFailed)?;

        debug!("Vulkan instance created successfully.");

        let mut i = Instance {
            entry: ManuallyDrop::new(entry),
            instance: ManuallyDrop::new(instance),
            debug_messenger: None,
        };

        #[cfg(debug_assertions)]
        {
            if !i.make_debug_messenger() {
                return Err(InstanceCreationError::FailedToCreateDebugMessenger);
            }
            debug!("Vulkan debug messenger created.");
        }

        Ok(i)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SurfaceError {
    #[error("Failed to get the window's display handle.")]
    FailedToGetDisplayHandle,
    #[error("Failed to get the window's window handle.")]
    FailedToGetWindowHandle,
    #[error("Failed to create Vulkan surface: {0}")]
    Creation(#[from] ash::vk::Result),
}

impl crate::core::Surface {
    pub(crate) fn new(
        instance: &crate::core::Instance,
        window: &winit::window::Window,
    ) -> Result<crate::core::Surface, SurfaceError> {
        let display_handle = window
            .display_handle()
            .map_err(|_| SurfaceError::FailedToGetDisplayHandle)?
            .as_raw();
        let window_handle = window
            .window_handle()
            .map_err(|_| SurfaceError::FailedToGetWindowHandle)?
            .as_raw();

        let surface = unsafe {
            ash_window::create_surface(
                &instance.entry,
                &instance.instance,
                display_handle,
                window_handle,
                None,
            )
        }
        .map_err(SurfaceError::Creation)?;
        let loader = ash::khr::surface::Instance::new(instance.as_ref(), instance);
        Ok(Self { surface, loader })
    }
}
