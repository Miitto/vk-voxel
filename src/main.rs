use tracing::{error, info, log::LevelFilter};
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .init();

    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app_wrapper = AppWrapper::default();

    event_loop.run_app(&mut app_wrapper);
}

#[derive(Default)]
struct AppWrapper {
    a: Option<App>,
}

struct App {
    window: winit::window::Window,
    vkcore: vk::core::VkCore,
}

impl winit::application::ApplicationHandler for AppWrapper {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let info = vk::AppInfo {
            name: "Voxel Testing",
            version: 1,
        };
        info!(
            "Starting application '{}', version {}.",
            info.name, info.version
        );
        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_title("Voxel Testing")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0)),
        ) {
            Ok(w) => w,
            Err(e) => {
                error!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        let vkcore = match vk::init(info, &window) {
            Ok(core) => core,
            Err(e) => {
                error!("Failed to initialize Vulkan Core: {e}");
                event_loop.exit();
                return;
            }
        };

        self.a = Some(App { window, vkcore });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        self.a.as_mut().unwrap().window_event(event_loop, id, event)
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested. Exiting application.");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                info!("Window resized to {}x{}.", new_size.width, new_size.height);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                info!("Keyboard input: {:?}", event.physical_key);
            }
            WindowEvent::RedrawRequested => {}
            _ => {}
        }
    }
}
