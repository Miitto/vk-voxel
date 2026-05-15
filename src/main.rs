use tracing::{error, info, log::LevelFilter};

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .init();

    let app_info = vk::AppInfo {
        name: "Voxel Testing",
        version: 1,
    };
    info!(
        "Starting application '{}', version {}.",
        app_info.name, app_info.version
    );

    let vkcore = {
        match vk::init(app_info) {
            Ok(core) => core,
            Err(e) => {
                error!("Failed to initialize Vulkan Core: {e}");
                return;
            }
        }
    };
}
