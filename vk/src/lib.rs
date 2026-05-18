pub mod core;
mod setup;

#[cfg(debug_assertions)]
mod validation;

pub use setup::AppInfo;

pub const FRAMES_IN_FLIGHT: usize = 2;

pub mod prelude {
    pub use ash::vk;
}

pub fn init(
    app_info: AppInfo,
    window: &winit::window::Window,
) -> Result<core::VkCore, setup::SetupError> {
    core::VkCore::new(app_info, window)
}

pub trait ToCBytes {
    fn to_cbytes(&self) -> Vec<*const std::ffi::c_char>;
}

impl ToCBytes for &[String] {
    fn to_cbytes(&self) -> Vec<*const std::ffi::c_char> {
        self.iter()
            .map(|s| {
                std::ffi::CString::new(s.as_str())
                    .expect("Failed to convert str to cstr")
                    .into_raw() as *const std::ffi::c_char
            })
            .collect()
    }
}

impl ToCBytes for [String] {
    fn to_cbytes(&self) -> Vec<*const std::ffi::c_char> {
        self.iter()
            .map(|s| {
                std::ffi::CString::new(s.as_str())
                    .expect("Failed to convert str to cstr")
                    .into_raw() as *const std::ffi::c_char
            })
            .collect()
    }
}

impl ToCBytes for &[&str] {
    fn to_cbytes(&self) -> Vec<*const std::ffi::c_char> {
        self.iter()
            .map(|s| {
                std::ffi::CString::new(*s)
                    .expect("Failed to convert str to cstr")
                    .into_raw() as *const std::ffi::c_char
            })
            .collect()
    }
}

impl ToCBytes for [&str] {
    fn to_cbytes(&self) -> Vec<*const std::ffi::c_char> {
        self.iter()
            .map(|s| {
                std::ffi::CString::new(*s)
                    .expect("Failed to convert str to cstr")
                    .into_raw() as *const std::ffi::c_char
            })
            .collect()
    }
}

impl ToCBytes for &[&std::ffi::CStr] {
    fn to_cbytes(&self) -> Vec<*const std::ffi::c_char> {
        self.iter().map(|s| s.as_ptr()).collect()
    }
}

impl ToCBytes for [&std::ffi::CStr] {
    fn to_cbytes(&self) -> Vec<*const std::ffi::c_char> {
        self.iter().map(|s| s.as_ptr()).collect()
    }
}
