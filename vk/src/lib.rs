mod core;
mod setup;

#[cfg(debug_assertions)]
mod validation;

pub use setup::AppInfo;

pub fn init(app_info: AppInfo) -> Result<core::VkCore, setup::SetupError> {
    core::VkCore::new(app_info)
}
